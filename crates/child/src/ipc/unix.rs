use child_ipc::{Error, Response};

pub fn send_response(resp: Result<Response, Error>) {
    let s = serde_json::to_string(&resp).unwrap();
    println!("{s}");
}
