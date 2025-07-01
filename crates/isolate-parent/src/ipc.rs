use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
};

use isolate_ipc::{Command, Error, Response};

pub(crate) fn call_child_process(command: Command) -> Result<Response, Error> {
    spawn_child_process(command)
        .map_err(|e| Error::Ipc(e.to_string()))?
        .0
        .next()
        .ok_or(Error::Ipc(
            "Unexpected response from child process".to_string(),
        ))?
}

pub(crate) fn spawn_child_process(
    command: Command,
) -> io::Result<(
    impl Iterator<Item = Result<Response, Error>>,
    impl FnOnce() -> io::Result<()>,
)> {
    let child_path = find_isolate_child().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "isolate-child executable not found",
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

    let mut child_for_exit = child;
    let exit_signal = move || {
        // Dropping stdin signals EOF to the child
        drop(child_for_exit.stdin.take());
        // Optionally wait for the child to exit
        child_for_exit.wait().map(|_| ())
    };

    Ok((iter, exit_signal))
}

fn find_isolate_child() -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        "isolate-child.exe"
    } else {
        "isolate-child"
    };

    // 1. Next to current executable
    if let Ok(current_exe) = env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let candidate = dir.join(exe_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 2. In PATH
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            let candidate = path.join(exe_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 3. In target/debug/ or target/release/ (for development)
    if let Ok(current) = env::current_dir() {
        let current_target = current.join("target");

        let debug_candidate = current_target.join("debug").join(exe_name);
        if debug_candidate.exists() {
            return Some(debug_candidate);
        }

        let release_candidate = current_target.join("release").join(exe_name);
        if release_candidate.exists() {
            return Some(release_candidate);
        }
    }

    None
}
