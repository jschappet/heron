use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage};
use futures_util::future::{LocalBoxFuture, ready, Ready};
use std::rc::Rc;
use crate::db::DbPool;
use crate::services::hosts::HostsService;
//use crate::middleware::host::{HostContext, HostInfo};

#[derive(Clone, Debug)]
pub struct HostInfo {
    pub id: i32,
    pub slug: String,
    pub host_name: String,
    pub display_name: String,
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct HostContext(pub Rc<HostInfo>);


pub struct HostMiddleware {
    pool: DbPool,
}

impl HostMiddleware {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

pub struct HostMiddlewareService<S> {
    inner: Rc<S>,
    pool: DbPool,
}

impl<S, B> Transform<S, ServiceRequest> for HostMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = HostMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HostMiddlewareService {
            inner: Rc::new(service),
            pool: self.pool.clone(),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for HostMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(ctx)
    }

    fn call(&self,  req: ServiceRequest) -> Self::Future {
        let pool = self.pool.clone();
        let inner = self.inner.clone(); // Rc clone
        log::debug!("HostMiddleware: Processing request for {}", req.path());
        // Owned host_header
        let host_header: String = match req
            .headers()
            .get("X-Forwarded-Host")
            .or_else(|| req.headers().get("Host"))
            .ok_or_else(|| actix_web::error::ErrorBadRequest("No Host header found"))
            .and_then(|h| h.to_str().map(|s| s.to_owned()).map_err(|_| {
                actix_web::error::ErrorBadRequest("Invalid Host header")
            }))
        {
            Ok(s) => s,
            Err(e) => return Box::pin(async { Err(e) }),
        };

        Box::pin(async move {
            log::debug!("HostMiddleware: Extracted host header: {}", host_header);
            let hosts_service = HostsService::new(pool);
            let host_info = hosts_service
                .get_host_by_name(&host_header)
                .map_err(actix_web::error::ErrorInternalServerError)?;

            let host_ctx = HostContext(Rc::new(HostInfo {
                id: host_info.id,
                slug: host_info.slug,
                host_name: host_info.host_name,
                display_name: host_info.display_name,
                base_url: host_info.base_url,
            }));

            req.extensions_mut().insert(host_ctx);

            inner.call(req).await
        })
    }
}
