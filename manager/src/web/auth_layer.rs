use core::fmt;

use axum::http::Request;
use axum::response::Response;
use tower_layer::Layer;
use tower_service::Service;

use crate::web::webstate::WebState;

#[derive(Clone)]
pub struct AuthLayer {
    webstate: WebState
}

impl AuthLayer {
    pub fn new(webstate: WebState) -> Self {
        Self { webstate }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthService {
            service
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    service: S,
}

impl<S, ReqBody, RspBody> Service<Request<ReqBody>> for AuthService<S> where 
    S: Service<Request<ReqBody>, Response = Response<RspBody>>,
    RspBody: Default
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // self.service.call()

        
        
        let uri = req.uri().to_string();
        println!("url = {:?}", uri);

        self.service.call(req)
    }
}