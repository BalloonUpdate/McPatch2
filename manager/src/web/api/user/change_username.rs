use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 新用户名
    new_username: String,
}

pub async fn api_change_username(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let mut config = state.config.config.lock().await;
    let mut token = state.token.lock().await;

    // 修改用户名
    config.web.username = payload.new_username;
    drop(config);
    state.config.save_async().await;

    // 使token失效
    token.clear();

    PublicResponseBody::<()>::ok_no_data()
}