use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 旧密码
    old_password: String,

    /// 新密码
    new_password: String,
}

pub async fn api_change_password(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let mut config = state.config.config.lock().await;
    let mut token = state.token.lock().await;

    if !config.web.test_password(&payload.old_password) {
        return PublicResponseBody::<()>::err("incorrect current password");
    }

    // 修改密码
    config.web.password = payload.new_password;
    drop(config);
    state.config.save_async().await;

    // 使token失效
    token.clear();

    PublicResponseBody::<()>::ok_no_data()
}