use child_ipc::{ChildError, Response};

pub fn send_response(resp: Result<Response, ChildError>) {
    let s = serde_json::to_string(&resp).unwrap();
    println!("{s}");
}
