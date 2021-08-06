use web_view::Content;

pub fn web_view() {
    let html = format!(
        r#"
        <!doctype html>
        <html>
            <body>
                <button onclick="window.webkit.messageHandlers.external.postMessage('exit')">foo</button>
                <script type="text/javascript">
                {}
                </script>
            </body>
        </html>"#,
        include_str!("index.js")
    );

    web_view::builder()
        .title("Ip Map")
        .content(Content::Html(html))
        .size(640, 480)
        .resizable(true)
        .debug(true)
        .invoke_handler(|webview, arg| {
            // match arg {
            //     "exit" => webview.eval("console.log('die')").unwrap(),
            //     _ => unimplemented!(),
            // };
            Ok(())
        })
        .user_data(())
        .run()
        .unwrap();
}
