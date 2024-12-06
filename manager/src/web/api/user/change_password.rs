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
    let mut auth = state.auth;

    if !auth.test_password(&payload.old_password).await {
        return PublicResponseBody::<()>::err("incorrect current password");
    }

    // 修改密码
    auth.set_password(&payload.new_password).await;
    auth.save().await;

    // 使token失效
    auth.clear_token().await;

    PublicResponseBody::<()>::ok_no_data()
}