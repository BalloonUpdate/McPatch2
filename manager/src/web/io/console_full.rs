use axum::body::Body;
use axum::extract::State;
use axum::response::Response;

use crate::web::webstate::WebState;

pub async fn api_console_full(State(state): State<WebState>) -> Response {
    let mut console = state.console.lock().await;

    let mut buf = String::with_capacity(1024);

    for log in console.get_logs(true) {
        buf += &log.content;
        buf += "\n";
    }

    Response::builder()
        .body(Body::new(buf))
        .unwrap()
}