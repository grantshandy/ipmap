use web_view::Content;

use crate::ip::IP_JSON_DOCUMENT;
use crate::ip::IP_INDEX;

pub fn web_view() {
    let html = format!(
        r#"
        <!doctype html>
        <html>
            <head>
            <link rel="stylesheet" href="https://unpkg.com/leaflet@1.7.1/dist/leaflet.css" integrity="sha512-xodZBNTC5n17Xt2atTPuE1HxjVMSvLVW9ocqUKLsCC5CXdbqCmblAshOMAS6/keqq/sMZMZ19scR4PsZChSR7A==" crossorigin=""/>
            <script src="https://unpkg.com/leaflet@1.7.1/dist/leaflet.js" integrity="sha512-XQoYMqMTK8LvdxXYG3nZ448hOEQiglfqkJs1NOQV44cWnUrBc8PkAOcXy20w0vlaXaVUearIOBhiXZ5V3ynxwA==" crossorigin=""></script>        
            <style type="text/css" media="screen">
            .container {{
                position:fixed;
                padding:0;
                margin:0;
                top:0;
                left:0;
                width: 100%;
                height: 100%;
            }}
            </style>
            </head>
            <body>
                <div id="mapid" class="container""></div>
                <script type="text/javascript">
                {}
                </script>
            </body>
        </html>"#,
        include_str!("index.js")
    );

    let mut is_fullscreen = false;

    println!("Starting UI");
    web_view::builder()
        .title("Ipmap")
        .content(Content::Html(html))
        .size(800, 600)
        .resizable(true)
        .debug(false)
        .invoke_handler(|webview, arg| {
            // This is the only place I have access to the webview variable... and it's called when JS calls something...
            // This means that I have to run a loop in JS that requests Rust to run a javascript function.
            // It's horrible for performance, but it's the only way to do it without creating a webserver and using websockets or something :/
            
            match arg {
                "requestData" => {
                    match IP_INDEX.read().unwrap().len() {
                        0 => webview.set_title("Ipmap").unwrap(),
                        1 => webview.set_title("Ipmap - 1 Connection").unwrap(),
                        _ => webview.set_title(&format!("Ipmap - {} Connections", IP_INDEX.read().unwrap().len())).unwrap(),
                    }

                    webview.eval(&format!("addMarkers({})", IP_JSON_DOCUMENT.read().unwrap())).unwrap();
                }
                "exitFullscreen" => {
                    webview.set_fullscreen(false);
                    is_fullscreen = false;
                }
                "toggleFullscreen" => {
                    match is_fullscreen {
                        true => {
                            is_fullscreen = false;
                            webview.set_fullscreen(false);
                        }
                        false => {
                            is_fullscreen = true;
                            webview.set_fullscreen(true);
                        }
                    }
                }
                "quit" => {
                    println!("Quitting!");
                    webview.exit();
                }
                "credits" => {
                    web_view::builder()
                        .title("Credits")
                        .content(Content::Html(include_str!("credits.html")))
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
