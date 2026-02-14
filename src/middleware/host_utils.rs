// src/middleware/host_utils.rs
use actix_web::HttpMessage;

use actix_web::{HttpRequest, HttpResponse};
use crate::middleware::host::{HostContext, HostInfo};
use std::rc::Rc;

/// Async-friendly helper: get the HostContext from the request extensions
/// Returns a cloned `Rc<HostInfo>` or an HTTP error if missing
pub async fn require_host(req: &HttpRequest) -> Result<Rc<HostInfo>, HttpResponse> {
    match req.extensions().get::<HostContext>() {
        Some(ctx) => Ok(Rc::clone(&ctx.0)),
        None => {
            log::warn!("Request missing HostContext in extensions");
            Err(HttpResponse::BadRequest().body("No HostContext found"))
        }
    }
}

/// Async-friendly helper: get the host ID only
pub async fn require_host_id(req: &HttpRequest) -> Result<i32, HttpResponse> {
    let host = require_host(req).await?;
    Ok(host.id)
}

/// Async-friendly helper: get the host slug only
pub async fn require_host_slug(req: &HttpRequest) -> Result<String, HttpResponse> {
    let host = require_host(req).await?;
    Ok(host.slug.clone())
}
