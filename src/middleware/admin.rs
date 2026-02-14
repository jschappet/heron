use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{types::MemberRole, validator::AuthContext};

// --- Your middleware marker type ---
pub struct AdminMiddleware;

// --- Transform implementation ---
impl<S, B> Transform<S, ServiceRequest> for AdminMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AdminMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminMiddlewareService { service }))
    }
}

// --- The actual service wrapper ---
pub struct AdminMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AdminMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth = req.extensions().get::<AuthContext>().cloned();
        let fut = self.service.call(req);

        Box::pin(async move {
            match auth {
                Some(auth) => {
                    let is_admin = auth
                        .memberships
                        .iter()
                        .any(|m| matches!(m.role, MemberRole::Admin));

                    if !is_admin {
                        return Err(ErrorUnauthorized("Not admin"));
                    }

                    fut.await
                }
                None => Err(ErrorUnauthorized("No auth")),
            }
        })
    }
}
