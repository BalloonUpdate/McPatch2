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
    let mut auth = state.auth;

    // 修改用户名
    auth.set_username(&payload.new_username).await;
    
    // 使token失效
    auth.clear_token().await;
    
    auth.save().await;

    PublicResponseBody::<()>::ok_no_data()
}