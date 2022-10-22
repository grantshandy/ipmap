use actix_web::{get, http::header::ContentType, web::Bytes, HttpResponse, Responder};

/// vitejs is setup with a plugin so that all html/js/css is put into a single file for convinience as seen here.
#[get("/")]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../frontend/dist/index.html"))
}

#[get("/marker-icon.png")]
pub async fn marker_icon() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(Bytes::from_static(include_bytes!(
            "../frontend/node_modules/leaflet/dist/images/marker-icon.png"
        )))
}

#[get("/marker-shadow.png")]
pub async fn marker_shadow() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(Bytes::from_static(include_bytes!(
            "../frontend/node_modules/leaflet/dist/images/marker-shadow.png"
        )))
}
