use std::net::IpAddr;

use ipgeo_state::LookupInfo;
use serde::{Deserialize, Serialize};
use specta::Type;

pub mod commands {
    use std::{net::IpAddr, sync::mpsc, thread};

    use ipgeo_state::DbState;
    use tauri::{State, ipc::Channel};
    use trippy_core::{Builder, ProbeStatus};

    use super::{Hop, TraceroutePreferences};

    #[tauri::command]
    #[specta::specta]
    pub async fn run_traceroute(
        state: State<'_, DbState>,
        prefs: TraceroutePreferences,
        round_update: Channel<usize>,
    ) -> Result<Vec<Hop>, String> {
        let (tx, rx) = mpsc::channel::<Option<usize>>();

        // round_update can't be sent into the catch_unwind,
        // but we can do this little trick.
        thread::spawn(move || {
            while let Ok(Some(u)) = rx.recv() {
                let _ = round_update.send(u);
            }
        });

        let snapshot = std::panic::catch_unwind(|| {
            let tracer = Builder::new(prefs.ip)
                .max_rounds(Some(prefs.max_rounds))
                .build()
                .unwrap();

            tracer
                .run_with(|round| {
                    let round: Option<usize> = round
                        .probes
                        .iter()
                        .filter_map(|status| match status {
                            ProbeStatus::Awaited(a) => Some(a.round.0),
                            ProbeStatus::Complete(c) => Some(c.round.0),
                            _ => None,
                        })
                        .max();

                    if let Some(round) = round {
                        let _ = tx.send(Some(round));
                    }
                })
                .unwrap();

            tracer.snapshot()
        });

        let snapshot = match snapshot {
            Ok(s) => s,
            Err(e) => {
                let e = format!("{e:?}");
                tracing::error!("{e}");
                return Err(e);
            }
        };

        let hops = snapshot
            .hops()
            .iter()
            // Filter hops to only global IPs
            .map(|h| h.addrs().copied().collect::<Vec<IpAddr>>())
            // Collect each IP hop with a location
            .map(|ips| {
                let location = ips
                    .iter()
                    .find_map(|ip| ipgeo_state::commands::lookup_ip(state.clone(), *ip));

                Hop { ips, location }
            })
            // Deduplicate hops by location, folding together IP fields
            .fold(Vec::new(), |mut acc: Vec<Hop>, current_hop: Hop| {
                if let Some(last_hop) = acc.last_mut() {
                    if last_hop.location == current_hop.location {
                        // Same location, fold IP addresses
                        last_hop.ips.extend(current_hop.ips);
                    } else {
                        // Different location, push as a new hop
                        acc.push(current_hop);
                    }
                } else {
                    // First hop, just push it
                    acc.push(current_hop);
                }

                acc
            });

        Ok(hops)
    }
}

#[derive(Clone, Debug, Type, Serialize, Deserialize)]
pub struct Hop {
    ips: Vec<IpAddr>,
    location: Option<LookupInfo>,
}

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceroutePreferences {
    ip: IpAddr,
    max_rounds: usize,
}
