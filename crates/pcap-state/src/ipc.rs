use std::{
    env,
    io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Command as ProcessCommand, Stdio},
};

use crate::StopCallback;
use child_ipc::{Command, Error, Response};

#[cfg(not(target_os = "windows"))]
const EXE_NAME: &str = "ipmap-child";

#[cfg(target_os = "windows")]
const EXE_NAME: &str = "ipmap-child.exe";

pub fn call_child_process(command: Command) -> Result<Response, Error> {
    tracing::debug!("calling {EXE_NAME} with {command:?}");

    let (mut child, mut reader) =
        spawn_child_process(command).map_err(|e| Error::Ipc(e.to_string()))?;

    let mut output: Vec<u8> = Vec::new();
    reader
        .read_to_end(&mut output)
        .map_err(|e| Error::Ipc(e.to_string()))?;

    let response: Result<Response, Error> =
        serde_json::from_slice(&output).map_err(|e| Error::Ipc(e.to_string()))?;

    if child
        .wait()
        .map_err(|e| Error::Ipc(e.to_string()))
        .is_ok_and(|e| e.success())
    {
        response
    } else {
        Err(Error::Ipc(format!("{EXE_NAME} returned nonzero exit code")))
    }
}

pub(crate) fn spawn_child_iterator(
    command: Command,
) -> io::Result<(impl Iterator<Item = Result<Response, Error>>, StopCallback)> {
    tracing::debug!("spawning {EXE_NAME} with {command:?}");

    let (mut child, reader) = spawn_child_process(command)?;

    // Process should only emit Result<Response, Error> as JSON strings separated by newlines.
    let iter = reader
        .lines()
        .map(|line| line.map_err(|e| Error::Ipc(e.to_string())))
        .map(|line| {
            line.and_then(
                |l| match serde_json::from_str::<Result<Response, Error>>(&l) {
                    Ok(resp) => resp,
                    Err(err) => Err(Error::Ipc(err.to_string())),
                },
            )
        });

    let exit_signal = Box::new(move || {
        child.kill()?;
        child.wait()?;
        tracing::debug!("{EXE_NAME} finished exiting");
        Ok(())
    });

    Ok((iter, exit_signal))
}

fn spawn_child_process(command: Command) -> io::Result<(Child, BufReader<ChildStdout>)> {
    let child_path = find_isolate_child().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        format!("{EXE_NAME} not found"),
    ))?;

    let mut child = ProcessCommand::new(child_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write the command as JSON to child's stdin
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(serde_json::to_string(&command).unwrap().as_bytes())?;
        stdin.write_all(b"\n")?;
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to open stdin"));
    }

    let Some(stdout) = child.stdout.take() else {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to get stdout"));
    };

    let reader = BufReader::new(stdout);

    Ok((child, reader))
}

fn find_isolate_child() -> Option<PathBuf> {
    if let Ok(env) = env::var("IPMAP_CHILD") {
        let candidate = Path::new(&env);

        if candidate.exists() {
            return Some(candidate.to_path_buf());
        } else {
            tracing::warn!("{EXE_NAME} '{candidate:?}' doesn't exist, not using.");
        }
    }

    // 1. Next to current executable
    if let Ok(current_exe) = env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let candidate = dir.join(EXE_NAME);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 2. In PATH
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            let candidate = path.join(EXE_NAME);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 3. In target/debug/ or target/release/ (for development)
    if let Ok(current) = env::current_dir() {
        let target = current.join("target");

        let debug = target.join("debug").join(EXE_NAME);
        if debug.exists() {
            return Some(debug);
        }

        let release = target.join("release").join(EXE_NAME);
        if release.exists() {
            return Some(release);
        }
    }

    None
}
