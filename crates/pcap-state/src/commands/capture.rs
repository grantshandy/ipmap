use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    time::Instant,
};

use child_ipc::{Command, Connection, Connections, Error, ErrorKind, Response, RunCapture, ipc};
use ipgeo::{Coordinate, DatabaseTrait, Location};
use ipgeo_state::DbState;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State, ipc::Channel};

use crate::{PcapState, PcapStateChange, PcapStateInfo};

#[tauri::command]
#[specta::specta]
pub async fn start_capture(
    app: AppHandle,
    pcap: State<'_, PcapState>,
    db: State<'_, DbState>,
    params: RunCapture,
    conns: Channel<CaptureLocations>,
) -> Result<(), Error> {
    let device = params.device.clone();

    let child_path = crate::resolve_child_path(app.path())?;
    let (child, exit) = ipc::spawn_child_process(child_path, Command::Capture(params)).await?;

    pcap.set_capture(device, exit);
    PcapStateChange::emit(&app).await;

    while let Ok(resp) = child.recv() {
        match resp {
            Ok(Response::CaptureSample(c)) => {
                let prev = Instant::now();
                let _ = conns.send(CaptureLocations::new(&db, c));
                tracing::info!(
                    "CaptureLocations::new took {}ms",
                    prev.elapsed().as_millis()
                );
            }
            Ok(_) => {
                pcap.stop_capture();
                return Err(Error::basic(ErrorKind::UnexpectedType));
            }
            Err(e) => {
                pcap.stop_capture();
                return Err(e);
            }
        }
    }

    pcap.stop_capture();
    let _ = conns.send(CaptureLocations::last());
    PcapStateChange::emit(&app).await;

    Ok(())
}

/// Stop the current capture.
#[tauri::command]
#[specta::specta]
pub async fn stop_capture(pcap: State<'_, PcapState>) -> Result<(), Error> {
    match pcap.stop_capture() {
        Some(Err(e)) => Err(Error::from(e)),
        _ => Ok(()),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn init_pcap(
    app: AppHandle,
    state: State<'_, PcapState>,
) -> Result<PcapStateInfo, Error> {
    state.info(app).await
}

#[derive(Debug, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaptureLocations {
    /// The current state of locations and their connections.
    pub updates: HashMap<CoordKey, CaptureLocation>,
    /// Coordinates that were created or had IPs added/destroyed.
    /// Indices into the updates fields.
    pub connections_changed: HashSet<CoordKey>,
    /// Connections that we couldn't find in the ip-geo database.
    pub not_found: HashMap<IpAddr, Connection>,
    /// A single Connection representing the entire capture session.
    pub session: Connection,
    /// The maximum connection throughput found.
    pub max_throughput: f64,
    /// Indicate to the client this is the last update in the session.
    pub last: bool,
}

impl CaptureLocations {
    pub fn new(db: &DbState, conn: Connections) -> Self {
        let max_throughput = conn
            .updates
            .iter()
            .map(|(_, info)| info.throughput())
            .reduce(f64::max)
            .unwrap_or(0.0);

        // combined coordinates of started/ended
        let connections_changed = conn
            .started
            .into_iter()
            .chain(conn.ended.into_iter())
            .filter_map(|ip| db.get_coordinate(ip))
            .map(coord_key)
            .collect();

        let mut updates_by_coord =
            HashMap::<Coordinate, HashMap<IpAddr, Connection>>::with_capacity(conn.updates.len());
        let mut not_found = HashMap::new();

        for (ip, info) in conn.updates {
            let Some(crd) = db.get_coordinate(ip) else {
                not_found.insert(ip, info);
                continue;
            };

            updates_by_coord
                .entry(crd)
                .or_insert(HashMap::new())
                .insert(ip, info);
        }

        let updates = updates_by_coord
            .into_iter()
            .map(|(crd, ips)| {
                let (up_s, down_s) = ips
                    .iter()
                    .map(|(_, c)| (c.up.avg_s, c.down.avg_s))
                    .fold((0.0, 0.0), |(ua, da), (u, d)| (ua + u, da + d));

                (
                    coord_key(crd),
                    CaptureLocation {
                        ips,
                        // UNWRAP: db.get_location returns Some if db.get_coordinate returns Some.
                        loc: db.get_location(crd).unwrap(),
                        crd,
                        dir: ConnectionDirection::new(up_s, down_s),
                        thr: up_s + down_s,
                    },
                )
            })
            .collect();

        Self {
            updates,
            connections_changed,
            not_found,
            session: conn.session,
            max_throughput,
            last: false,
        }
    }

    /// Indicate to the frontend UI that the capture session has just stopped.
    pub fn last() -> Self {
        Self {
            last: true,
            ..Default::default()
        }
    }
}

/// A location and it's associated active IPs and their connections.
#[derive(Debug, Serialize, Deserialize, Type)]
pub struct CaptureLocation {
    pub ips: HashMap<IpAddr, Connection>,
    pub loc: Location,
    pub crd: Coordinate,
    pub dir: ConnectionDirection,
    pub thr: f64,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionDirection {
    #[default]
    Mixed,
    Up,
    Down,
}

impl ConnectionDirection {
    pub fn new(up: f64, down: f64) -> Self {
        let ratio = f64::min(up, down) / f64::max(up, down);

        // TODO: mess with this, or make it changeable in the settings?
        if ratio > 0.7 {
            ConnectionDirection::Mixed
        } else if up > down {
            ConnectionDirection::Up
        } else {
            ConnectionDirection::Down
        }
    }
}

type CoordKey = String;

fn coord_key(Coordinate { lat, lng, .. }: Coordinate) -> CoordKey {
    format!("{lat}|{lng}")
}
