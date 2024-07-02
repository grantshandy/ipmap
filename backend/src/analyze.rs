use std::net::{IpAddr, Ipv4Addr};

#[tauri::command]
pub async fn dns_lookup_addr(ip: IpAddr) -> Option<String> {
    dns_lookup::lookup_addr(&ip).ok()
}

#[tauri::command]
pub async fn validate_ip(ip: String) -> bool {
    ip.parse::<Ipv4Addr>()
        .is_ok_and(|ip| ip_rfc::global_v4(&ip))
}

// #[cfg(windows)]
// #[tauri::command]
// pub async fn traceroute<R: Runtime>(handle: AppHandle<R>, ip: Ipv4Addr) -> Result<String, String> {
//     use std::thread;

//     use libtraceroute::{util::Protocol, Config, Traceroute};
//     use tauri::Manager;

//     let update_event_name = format!("{}-update", u32::from(ip));

//     let update_event_name_send = update_event_name.clone();
//     thread::spawn(move || {
//         tracing::info!("tracerouting {ip}");

//         Traceroute::new(ip, Config::default().with_protocol(Protocol::UDP))
//             .perform_traceroute()
//             .iter()
//             .flat_map(|hop| hop.query_result.last())
//             .flat_map(|ip| ip.addr.parse::<Ipv4Addr>().ok())
//             .for_each(|ip| {
//                 tracing::info!("got {ip}");

//                 handle
//                     .emit_all(&update_event_name_send, ip)
//                     .expect("send traceroute update");
//             });

//         handle
//             .emit_all(&update_event_name_send, ())
//             .expect("send traceroute update");
//         tracing::info!("finished traceroute");
//     });

//     Ok(update_event_name)
// }

// EXCEEDINGLY bad practice but the best option considering the main rust traceroute library uses String as it's IP type and no Option...
// #[cfg(windows)]
// #[tauri::command]
// pub async fn traceroute<R: Runtime>(handle: AppHandle<R>, ip: Ipv4Addr) -> Result<String, String> {
//     use std::{
//         io::{BufRead, BufReader},
//         process::{Command, Stdio},
//         thread,
//     };

//     use tauri::Manager;

//     fn parse_tracert_stdout_line(line: String) -> Option<Ipv4Addr> {
//         // first and last lines contain the word "Tracing"
//         if line.is_empty() || line.contains("Trac") {
//             return None;
//         }

//         line.split_ascii_whitespace()
//             .last()?
//             .chars()
//             .filter(|c| c.is_numeric() || *c == '.')
//             .collect::<String>()
//             .parse::<Ipv4Addr>()
//             .ok()
//     }

//     let mut cmd = Command::new("tracert")
//         .args(["-d", "-w", "100", "-4", ip.to_string().as_str()])
//         .stdout(Stdio::piped())
//         .spawn()
//         .map_err(|e| {
//             tracing::error!("{e}");
//             format!("Failed to run tracert: {e}")
//         })?;

//     let update_event_name = format!("{}-update", u32::from(ip));

//     let update_event_name_send = update_event_name.clone();
//     thread::spawn(move || {
//         let out_reader = BufReader::new(cmd.stdout.as_mut().expect("get cmd stdout"));

//         tracing::info!("tracerouting {ip}");

//         out_reader
//             .lines()
//             .flat_map(Result::ok)
//             .flat_map(parse_tracert_stdout_line)
//             .filter(ip_rfc::global_v4)
//             .for_each(|ip| {
//                 tracing::info!("got {ip:?}");
//                 handle
//                     .emit_all(&update_event_name_send, ip)
//                     .expect("send traceroute update");
//             });

//         cmd.wait().expect("wait for command");

//         handle
//             .emit_all(&update_event_name_send, ())
//             .expect("send traceroute update");
//         tracing::info!("finished traceroute");
//     });

//     Ok(update_event_name)
// }

// #[cfg(unix)]
// pub async fn traceroute<R: tauri::Runtime>(
//     handle: tauri::AppHandle<R>,
//     ip: std::net::Ipv4Addr,
// ) -> Result<String, String> {
//     Err("TODO!".to_string())
// }
