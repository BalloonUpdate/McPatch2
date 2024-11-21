use core::fmt;

use tower_layer::Layer;
use tower_service::Service;

pub struct AuthLayer {

}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthService {
            service
        }
    }
}

pub struct AuthService<S> {
    service: S,
}

impl<S, R> Service<R> for AuthService<S> where 
    S: Service<R>,
    R: fmt::Debug
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        // self.service.call()
        self.service.call(req)
    }
}