#![feature(ip)]
mod ui;
mod ip;

#[tokio::main]
async fn main() {
    std::thread::spawn(|| {
        ui::web_view();
    });

    ip::manage_ip().await;
}