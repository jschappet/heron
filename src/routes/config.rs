use actix_web::{HttpRequest, HttpResponse, Responder, Scope, get, web};
use serde::Serialize;

use crate::db::DbPool;
use crate::errors::app_error::AppError;
use crate::middleware::host_utils::{require_host, require_host_slug};
use crate::services::contribute_events::{ContributionDomain, ContributionEventsService};
use crate::types::DraftStatus;
use crate::types::{Difficulty, Dietary, ConfigOption};

#[derive(Serialize)]
pub struct ConfigHash {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub difficulty: Vec<ConfigOption>,
    pub dietary: Vec<ConfigOption>,
    pub draft_status: Vec<ConfigOption>,
    pub contexts: Vec<ConfigHash>,
    
}


#[get("")]
pub async fn get_config_api(
    contributions: web::Data<ContributionDomain>,
) -> Result<HttpResponse, AppError> {
    ;
    //let contribution = contributions.get_effort_contexts(); // You can handle errors as needed
    let contribution = match contributions.get_effort_contexts(crate::types::Audience::Public) {
        Ok(contribution) => contribution,
        Err(e) => return Err(e),
    };

    let config = ConfigResponse {
        draft_status: DraftStatus::all(),
        difficulty: Difficulty::all(),
        dietary: Dietary::all(),
        contexts: contribution, 
    };

    Ok(HttpResponse::Ok().json(config))
}

use serde_json::json;
use std::path::Path;

// Example DB and Mail modules; adapt to your code
mod db {
    pub fn ping() -> Result<(), ()> {
        // lightweight DB check, e.g. SELECT 1
        Ok(())
    }
}

mod mail {
    pub fn ping() -> Result<(), ()> {
        // lightweight SMTP check (connection test only)
        Ok(())
    }
}

use crate::build_info;

#[get("/ONLINE")]
async fn online() -> impl Responder {
    // Build info from env

    // System health checks
    let db_status = if db::ping().is_ok() { "ok" } else { "fail" };
    let mail_status = if mail::ping().is_ok() { "ok" } else { "fail" };
    let uploads_status = if Path::new("/shared/uploads").exists() { "ok" } else { "fail" };

    HttpResponse::Ok().json(json!({
        "build": build_info::BUILD_TIME,
        "build_date": build_info::BUILD_DATE,
        "status": {
            "db": db_status,
            "mail": mail_status,
            "uploads": uploads_status
        }
    }))
}


use crate::middleware::host::HostContext;


#[get("/ping")]
async fn ping(req: HttpRequest) -> HttpResponse {
    match require_host(&req).await {
        Ok(host) => {
            log::debug!("Ping from host: {} (ID: {})", host.display_name, host.id);
            HttpResponse::Ok().body(format!("Host ID: {}, Slug: {}", host.id, host.slug))
        }
        Err(resp) => resp,
    }
}




pub fn scope() -> Scope {
    web::scope("")
        .service(get_config_api)
        .service(online)
        .service(ping)
}