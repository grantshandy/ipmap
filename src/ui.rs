use web_view::Content;
use web_view::WebViewBuilder;

#[cfg(windows)]
use pcap::Device;

use crate::ip::IP_JSON_DOCUMENT;

pub fn web_view() {
    // what a mess... hey it works good whatever.
    let html = include_str!("web/index.html")
        .to_string()
        .replace("/* rust inserts css here */", include_str!("web/style.css"));

    // there's a difference between JS files because the web engine backends use different methods for invoking rust functions.
    #[cfg(windows)]
    let html = html.replace(
        "// rust inserts insert js here",
        include_str!("web/index.windows.js"),
    );

    #[cfg(unix)]
    let html = html.replace(
        "// rust inserts insert js here",
        include_str!("web/index.unix.js"),
    );

    let mut is_fullscreen = false;

    println!("Starting UI");
    WebViewBuilder::new()
        .title("Ipmap")
        .content(Content::Html(html))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .invoke_handler(|webview, arg| {
            // This is the only place I have access to the webview variable... and it's called when JS calls something...
            // This means that I have to run a loop in JS that requests Rust to run a javascript function.
            // It's horrible for performance, but it's the only way to do it without creating a webserver and using websockets or something :/

            match arg {
                "requestData" => {
                    match IP_JSON_DOCUMENT.read().unwrap().matches(",").count() / 5 {
                        0 => webview.set_title("Ipmap").unwrap(),
                        1 => webview.set_title("Ipmap - 1 Connection").unwrap(),
                        _ => webview
                            .set_title(&format!(
                                "Ipmap - {} Connections",
                                IP_JSON_DOCUMENT.read().unwrap().matches(",").count() / 5
                            ))
                            .unwrap(),
                    }

                    webview
                        .eval(&format!(
                            "addMarkers([{}])",
                            IP_JSON_DOCUMENT.read().unwrap()
                        ))
                        .unwrap();
                }
                "exitFullscreen" => {
                    webview.set_fullscreen(false);
                    is_fullscreen = false;
                }
                "toggleFullscreen" => match is_fullscreen {
                    true => {
                        is_fullscreen = false;
                        webview.set_fullscreen(false);
                    }
                    false => {
                        is_fullscreen = true;
                        webview.set_fullscreen(true);
                    }
                },
                "quit" => {
                    println!("Quitting!");
                    webview.exit();
                }
                "credits" => {
                    let html = include_str!("web/credits.html")
                        .to_string()
                        .replace("/* rust inserts css here */", include_str!("web/style.css"));

                    web_view::builder()
                        .title("Credits")
                        .content(Content::Html(html))
                        .size(350, 220)
                        .resizable(false)
                        .debug(false)
                        .user_data(())
                        .invoke_handler(|_webview, _arg| Ok(()))
                        .run()
                        .unwrap();
                }
                _ => (),
            }
            Ok(())
        })
        .user_data(())
        .run()
        .unwrap();

    std::process::exit(0);
}

#[cfg(windows)]
pub fn windows_select_device() -> Device {
    let html = include_str!("web/device_select.html")
        .to_string()
        .replace("/* rust inserts css here */", include_str!("web/style.css"));

    let mut devices = Device::list().unwrap();
    let mut device: Option<Device> = None;

    if devices.is_empty() {
        println!("Found no device to listen on, maybe you need to run as an Adminstrator");
        std::process::exit(1);
    }

    WebViewBuilder::new()
        .title("Select Capture Device")
        .content(Content::Html(html))
        .size(350, 400)
        .resizable(true)
        .debug(false)
        .user_data(())
        .invoke_handler(|webview, arg| {
            if arg == "init" {
                for (i, d) in devices.iter().enumerate() {
                    let js = format!(r#"document.body.innerHTML += "<p>{} - <button onclick=\"external.invoke(\'use-{}\')\" >Use Me</button></p>";"#, d.desc.clone().unwrap_or("Unknown Name".to_string()), i);
                    webview.eval(&js).unwrap();
                }
            }

            if let Some(data) = arg.split("-").nth(1) {
                webview.exit();
                device = Some(devices.remove(data.parse().unwrap()));
            };

            Ok(())
        })
        .build()
        .unwrap()
        .run()
        .unwrap();

    match device {
        Some(data) => data,
        None => {
            eprintln!("you must choose a device!");
            std::process::exit(1);
        }
    }
}
