use std::{iter, net::IpAddr};

use child_ipc::{Command, Error, ErrorKind, Response, RunTraceroute, ipc};
use ipgeo::LookupInfo;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Runtime, State, ipc::Channel};
use tauri_plugin_ipgeo::{DbState, commands::lookup_ip};

use crate::model::PcapState;

#[tauri::command]
#[specta::specta]
#[cfg_attr(windows, allow(unused_variables))]
pub async fn traceroute_enabled<R: Runtime>(app: AppHandle<R>) -> Result<(), Error> {
    #[cfg(windows)]
    return Ok(());

    #[cfg(not(windows))]
    {
        let path = crate::model::ensure_child_path(&app)?;

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
pub async fn run_traceroute<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    pcap: State<'_, PcapState>,
    params: RunTraceroute,
    progress: Channel<usize>,
) -> Result<Vec<Hop>, Error> {
    let child_path = crate::model::ensure_child_path(&app)?;

    let (child, exit) = ipc::spawn_child_process(child_path, Command::Traceroute(params)).await?;

    let exit = || exit().map_err(Error::from);

    loop {
        match child.recv()? {
            Ok(Response::Progress(round)) => {
                let _ = progress.send(round);
            }
            Ok(Response::Traceroute(resp)) => {
                exit()?;

                let (ip, loc) = pcap.my_location(&app).await;

                return Ok(iter::once(Hop {
                    ips: vec![ip],
                    loc: Some(loc),
                })
                .chain(resp.into_iter().map(|ips| Hop::new(ips, &db)))
                .collect::<Vec<_>>());
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
    pub fn new(ips: Vec<IpAddr>, db: &State<'_, DbState>) -> Self {
        let loc = ips.iter().find_map(|ip| lookup_ip(db.clone(), *ip));

        Self { ips, loc }
    }
}
