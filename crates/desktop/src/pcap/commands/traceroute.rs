use std::{iter, net::IpAddr};

use crate::db::{DbState, LookupInfo, commands::my_location};
use child_ipc::{Command, Error, ErrorKind, Response, RunTraceroute, ipc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State, ipc::Channel};

#[tauri::command]
#[specta::specta]
#[cfg_attr(windows, allow(unused_variables))]
pub async fn traceroute_enabled(app: AppHandle) -> Result<(), Error> {
    #[cfg(windows)]
    return Ok(());

    #[cfg(not(windows))]
    {
        let path = crate::pcap::resolve_child_path(app.path())?;

        match ipc::call_child_process(path.clone(), Command::TracerouteStatus).await? {
            Response::TraceStatus(true) => Ok(()),
            Response::TraceStatus(false) => Err(Error::message(
                ErrorKind::InsufficientPermissions,
                path.display().to_string(),
            )),
            _ => Err(Error::basic(ErrorKind::UnexpectedType)),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn run_traceroute(
    app: AppHandle,
    db: State<'_, DbState>,
    params: RunTraceroute,
    progress: Channel<usize>,
) -> Result<Vec<Hop>, Error> {
    let child_path = crate::pcap::resolve_child_path(app.path())?;

    let (child, exit) = ipc::spawn_child_process(child_path, Command::Traceroute(params)).await?;

    let exit = || exit().map_err(Error::from);

    loop {
        match child.recv()? {
            Ok(Response::Progress(round)) => {
                let _ = progress.send(round);
            }
            Ok(Response::Traceroute(resp)) => {
                exit()?;

                let my_location = match crate::db::my_loc::get().await {
                    Ok((ip, Some(info))) => Some(Hop {
                        ips: vec![ip],
                        loc: Some(info),
                    }),
                    Ok((ip, None)) => Some(Hop {
                        ips: vec![ip],
                        loc: my_location(db.clone()).await.ok(),
                    }),
                    Err(_) => None,
                };

                let hops = resp.into_iter().map(|ips| Hop::new(ips, db.clone()));

                return match my_location {
                    Some(me) => Ok(iter::once(me).chain(hops).collect()),
                    None => Ok(hops.collect()),
                };
            }
            Ok(_) => {
                exit()?;
                return Err(Error::basic(ErrorKind::UnexpectedType));
            }
            Err(e) => {
                exit()?;
                return Err(e);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
pub struct Hop {
    ips: Vec<IpAddr>,
    loc: Option<LookupInfo>,
}

impl Hop {
    pub fn new(ips: Vec<IpAddr>, db: State<'_, DbState>) -> Self {
        let loc = ips
            .iter()
            .find_map(|ip| crate::db::commands::lookup_ip(db.clone(), *ip));

        Self { ips, loc }
    }
}
