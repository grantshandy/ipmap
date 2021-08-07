use web_view::Content;

pub fn web_view() {
    let html = format!(
        r#"
        <!doctype html>
        <html>
            <head>
            <link rel="stylesheet" href="https://unpkg.com/leaflet@1.7.1/dist/leaflet.css" integrity="sha512-xodZBNTC5n17Xt2atTPuE1HxjVMSvLVW9ocqUKLsCC5CXdbqCmblAshOMAS6/keqq/sMZMZ19scR4PsZChSR7A==" crossorigin=""/>
            <script src="https://unpkg.com/leaflet@1.7.1/dist/leaflet.js" integrity="sha512-XQoYMqMTK8LvdxXYG3nZ448hOEQiglfqkJs1NOQV44cWnUrBc8PkAOcXy20w0vlaXaVUearIOBhiXZ5V3ynxwA==" crossorigin=""></script>        
            <style type="text/css" media="screen">
            {}
            </style>
            </head>
            <body>
                <div id="mapid" class="container""></div>
                <p id="totalIps"></p>
                <script type="text/javascript">
                {}
                </script>
            </body>
        </html>"#,
        include_str!("style.css"),
        include_str!("index.js")
    );

    web_view::builder()
        .title("Ip Map")
        .content(Content::Html(html))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .invoke_handler(|webview, arg| {
            // This is the only place I have access to the webview variable... and it's called when JS calls something...
            match arg {
                "rustFunc" => webview.eval(&format!("addMarkers({})", crate::ip::IP_JSON_DOCUMENT.read().unwrap())).unwrap(),
                _ => (),
            }
            Ok(())
        })
        .user_data(())
        .run()
        .unwrap();

    std::process::exit(1);
}
