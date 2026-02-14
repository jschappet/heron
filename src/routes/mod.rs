pub mod offers_api;

pub mod users_api;

pub mod roles_api;

pub mod memberships_api;

pub mod ratings_api;

pub mod config;

pub mod profile;

pub mod authentication;

pub mod drafts_api;
pub mod effort_context;
pub mod misc;

pub mod uploads;

pub mod contribution_event;
pub mod events_api;
pub mod mailing_list;
pub mod ticket_api;
pub mod twillio;
pub mod twillio_admin;
pub mod weekly_answers;

// routes/mod.rs
use actix_web::{Scope, web};


// helper to reduce boilerplate
fn scoped(path: &str, inner: Scope) -> Scope {
    web::scope(path).service(inner)
}

pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(scoped("/auth", authentication::scope()))
        .service(scoped("/users", users_api::scope()))
        .service(scoped("/config", config::scope()))
        .service(scoped("/roles", roles_api::scope()))
        .service(scoped("/profile", profile::scope()))
        .service(scoped("/memberships", memberships_api::scope()))
        .service(scoped("/offers", offers_api::scope()))
        .service(scoped("/drafts", drafts_api::scope()))
        .service(scoped("/ratings", ratings_api::scope()))
        .service(scoped("/events", events_api::scope()))
        .service(scoped("/upload", uploads::scope()))
        .service(scoped("/weekly-answers", weekly_answers::scope()))
        .service(scoped("/ticket", ticket_api::scope()))
        .service(scoped("/mail", mailing_list::scope()))
        .service(scoped("/celebrate", contribution_event::scope()))
        // Twilio integration example
        .service(scoped("/twillio", twillio::scope()))
        .service(scoped("/twillio-admin", twillio_admin::scope()))
        .service(scoped("/admin", admin_scope()))
}

pub fn admin_scope() -> Scope {
    
    web::scope("")
        .service(scoped("/contrib_context",     contribution_event::admin_scope()))
}
