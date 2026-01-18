use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use child_ipc::{
    Command, Device, EXE_NAME, Error, ErrorKind, Response,
    ipc::{self, StopCallback},
};
use ipgeo::Database;
use ipgeo::{Coordinate, Location, LookupInfo};
use public_ip_address::response::LookupResponse;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_ipgeo::DbState;
use tauri_specta::Event;

struct CaptureSession {
    stop: StopCallback,
    device: Device,
}

#[derive(Default)]
pub struct PcapState {
    capture: Arc<Mutex<Option<CaptureSession>>>,
    my_location: OnceLock<(IpAddr, LookupInfo)>,
}

impl PcapState {
    pub fn stop_capture(&self) -> Option<io::Result<()>> {
        match self.capture.lock().map(|mut guard| guard.take()) {
            Ok(Some(CaptureSession { stop, .. })) => Some(stop()),
            _ => None,
        }
    }

    pub fn set_capture(&self, device: Device, stop: StopCallback) {
        self.stop_capture();

        // TODO: unwrap
        self.capture
            .lock()
            .map(|mut g| g.replace(CaptureSession { device, stop }))
            .unwrap();
    }

    pub async fn info<R: Runtime>(&self, app: AppHandle<R>) -> Result<PcapStateInfo, Error> {
        let capture: Option<Device> = self
            .capture
            .lock()
            .ok()
            .and_then(|c| c.as_ref().map(|c| c.device.clone()));

        let status =
            match ipc::call_child_process(ensure_child_path(&app)?, Command::PcapStatus).await? {
                Response::PcapStatus(status) => Ok(status),
                _ => Err(Error::basic(ErrorKind::UnexpectedType)),
            }?;

        let (_, my_location) = self.my_location(&app).await;

        Ok(PcapStateInfo {
            version: status.version,
            devices: status.devices,
            capture,
            my_location,
        })
    }

    pub async fn my_location<R: Runtime>(&self, app: &AppHandle<R>) -> (IpAddr, LookupInfo) {
        match self.my_location.get() {
            Some(crd) => crd.clone(),
            None => {
                let crd = get_my_location(app).await;
                self.my_location.get_or_init(|| crd).clone()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct PcapStateInfo {
    /// The version information about the currently loaded libpcap
    version: String,
    /// The list of available network devices for capture
    devices: Vec<Device>,
    /// The currently-captured on device, if any
    capture: Option<Device>,
    /// Where the user is suspected to currently be.
    my_location: LookupInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
#[serde(tag = "status")]
pub enum PcapStateChange {
    Ok(PcapStateInfo),
    Err(Error),
}

impl PcapStateChange {
    pub async fn emit<R: Runtime>(app: &AppHandle<R>) {
        let info = match app.state::<PcapState>().inner().info(app.clone()).await {
            Ok(info) => Self::Ok(info),
            Err(err) => Self::Err(err),
        };

        let _ = info.emit(app);
    }
}

const CHILD_HASH: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ipmap-child.sha256"));

/// Get the path of the ipmap-child executable, copying it to the local app dir if it doesn't exist.
pub(crate) fn ensure_child_path<R: Runtime>(handle: &AppHandle<R>) -> Result<PathBuf, Error> {
    let child_path = handle
        .path()
        .app_local_data_dir()
        .map_err(|e| Error::runtime(e.to_string()))?
        .join(EXE_NAME);

    if !child_path.exists() {
        fs::write(&child_path, crate::child::CHILD_BYTES)
            .map_err(|e| Error::runtime(e.to_string()))?;
        tracing::debug!("copied child executable to {child_path:?}");
    } else {
        let file = File::open(&child_path).map_err(|e| Error::runtime(e.to_string()))?;
        let file_hash = sha256_reader(&file).map_err(|e| Error::runtime(e.to_string()))?;

        if file_hash != CHILD_HASH {
            fs::write(&child_path, crate::child::CHILD_BYTES)
                .map_err(|e| Error::runtime(e.to_string()))?;
            tracing::warn!("updated child executable at {child_path:?} due to hash mismatch");
        }
    }

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&child_path)
            .map_err(|e| Error::runtime(e.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&child_path, perms).map_err(|e| Error::runtime(e.to_string()))?;
    }

    Ok(child_path)
}

fn sha256_reader<R: Read + Seek>(mut reader: R) -> io::Result<Vec<u8>> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    reader.seek(SeekFrom::Start(0))?;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_vec())
}

async fn get_my_location<R: Runtime>(handle: &AppHandle<R>) -> (IpAddr, LookupInfo) {
    let default_value = || {
        (
            IpAddr::V4(Ipv4Addr::BROADCAST),
            LookupInfo {
                crd: Coordinate { lat: 0.0, lng: 0.0 },
                loc: Location {
                    city: None,
                    region: None,
                    country_code: "??".into(),
                },
            },
        )
    };

    match public_ip_address::perform_lookup(None).await {
        Ok(LookupResponse {
            ip,
            latitude: Some(lat),
            longitude: Some(lng),
            city,
            region,
            country_code: Some(country_code),
            ..
        }) => (
            ip,
            LookupInfo {
                crd: Coordinate {
                    lat: lat as f32,
                    lng: lng as f32,
                },
                loc: Location {
                    city,
                    region,
                    country_code,
                },
            },
        ),
        Ok(LookupResponse { ip, .. }) => handle
            .state::<DbState>()
            .get(ip)
            .map(|loc| (ip, loc))
            .unwrap_or_else(|| {
                tracing::error!(
                    "Failed to get location for {ip}, your location will default to 0,0"
                );
                default_value()
            }),
        Err(err) => {
            tracing::error!("Failed to get location: {err}, your location will default to 0,0");
            default_value()
        }
    }
}
