use axum::extract::State;
use axum::response::Response;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

pub async fn api_check_token(State(_state): State<WebState>) -> Response {
    PublicResponseBody::<()>::ok_no_data()
}