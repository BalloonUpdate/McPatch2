use axum::body::Body;
use axum::response::Response;
use serde::Serialize;

pub mod user;
pub mod task;
pub mod fs;
pub mod terminal;
pub mod public;
pub mod webpage;
pub mod misc;

/// 公共响应体
#[derive(Serialize)]
pub struct PublicResponseBody<T> where T : Serialize {
    /// 状态码，1代表成功，其它值则代表失败
    pub code: i32,

    /// 附带的消息，通常在失败的时候用来说明原因
    pub msg: String,

    /// 返回的数据，仅当请求成功时有值，失败则为null
    /// 部分接口可能没有data部分，但是code仍要进行检查和弹出toast提示
    pub data: Option<T>,
}

impl<T> PublicResponseBody<T> where T : Serialize {
    pub fn ok(data: T) -> Response {
        Self {
            code: 1,
            msg: "ok".to_owned(),
            data: Some(data),
        }.to_response()
    }

    pub fn ok_no_data() -> Response {
        Self {
            code: 1,
            msg: "ok".to_owned(),
            data: None,
        }.to_response()
    }

    pub fn err(reason: &str) -> Response {
        Self {
            code: -1,
            msg: reason.to_owned(),
            data: None,
        }.to_response()
    }

    fn to_response(self) -> Response {
        let json = serde_json::to_string_pretty(&self).unwrap();

        Response::builder()
            .header(axum::http::header::CONTENT_TYPE, "application/json;charset=UTF-8")
            .body(Body::new(json))
            .unwrap()
    }
}
