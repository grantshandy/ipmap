use crate::{AppState, STREAM_KEEP_ALIVE_SECS};
use actix_web::{get, web, Responder};
use actix_web_lab::sse;
use log::{error, info};
use tokio::time::Duration;

#[get("/ip_stream")]
pub async fn ip_stream(data: web::Data<AppState>) -> impl Responder {
    info!("starting new connection");

    let mut stream_rx = data.into_inner().stream_rx.clone();
    let (sse_tx, sse) = sse::channel(2);

    tokio::spawn(async move {
        while stream_rx.changed().await.is_ok() {
            let msg = stream_rx.borrow().to_owned();

            if let Err(error) = sse_tx.send(sse::Data::new(msg)).await {
                error!("Error sending to client: {error}");
                break;
            }
        }
    });

    sse.with_keep_alive(Duration::from_secs(STREAM_KEEP_ALIVE_SECS))
}
