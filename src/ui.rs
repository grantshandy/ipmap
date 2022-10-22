use std::process;

use log::error;
use web_view::Content;

pub fn run_gui(ip: &String, port: &u16) {
    if let Err(error) = web_view::builder()
        .title("Ipmap")
        .content(Content::Url(&format!("http://{ip}:{port}/")))
        .size(800, 600)
        .resizable(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
    {
        error!("Error opening gui: {error}");
        process::exit(1);
    } else {
        process::exit(0);
    }
}
