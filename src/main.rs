use web_view::*;

fn main() {
    web_view::builder()
        .title("Ip Map")
        .content(Content::Html(include_str!("index.html")))
        .size(640, 480)
        .resizable(true)
        .debug(true)
        .invoke_handler(|_webview, arg| {
            match arg {
                "foo" => println!("bruh you fooed"),
                _ => unimplemented!(),
            };
            Ok(())
        })
        .user_data(())
        .run()
        .unwrap();


}