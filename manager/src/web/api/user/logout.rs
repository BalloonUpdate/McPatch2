use axum::extract::State;
use axum::response::Response;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

pub async fn api_logout(State(state): State<WebState>) -> Response {
    let _config = state.config.config.lock().await;

    let mut token = state.token.lock().await;

    token.clear();

    PublicResponseBody::<()>::ok_no_data()
}