use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 用户名
    username: String,

    /// 密码
    password: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    pub token: String,
}

pub async fn api_login(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let mut auth = state.auth;

    let ok = auth.test_username(&payload.username).await && auth.test_password(&payload.password).await;

    if !ok {
        return PublicResponseBody::<ResponseData>::err("incorrect username or password");
    }

    // 生成新的token
    let new_token = auth.regen_token().await;

    PublicResponseBody::<ResponseData>::ok(ResponseData { token: new_token })
}