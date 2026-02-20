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
pub mod twilio;
pub mod twilio_admin;
pub mod weekly_answers;

pub mod hosts;

// routes/mod.rs
use actix_web::{Scope, web};
use serde::Serialize;

use crate::{middleware::admin_middleware::AdminMiddleware, types::MemberRole};
use crate::types::method::Method;

//use crate::middleware::admin::AdminMiddleware;


// helper to reduce boilerplate
fn _scoped(path: &str, inner: Scope) -> Scope {
    web::scope(path).service(inner)
}

use std::cell::RefCell;

thread_local! {
    static SCOPE_CTX: RefCell<ScopeContext> = RefCell::new(ScopeContext::root());
}

fn role_allows(user_roles: &[MemberRole], route_roles: &[MemberRole]) -> bool {
    // public routes always allowed
    if route_roles.contains(&MemberRole::Public) {
        return true;
    }

    // if route requires any role, user must have one
    route_roles.iter().any(|r| user_roles.contains(r))
}



#[derive(Clone)]
pub struct ScopeContext {
    pub prefix: String,
    pub group: String,
    pub roles: MemberRole,
}

impl ScopeContext {
    fn root() -> Self {
        Self {
            prefix: "".into(),
            group: "".into(),
            roles: MemberRole::Public,
        }
    }
}



#[derive(Clone, Debug, Serialize)]
pub struct Route {
    pub parent_path: String,
    pub key: &'static str,
    pub segments: Vec<String>,
    pub method: &'static str,
    pub auth: bool,
    pub roles: Vec<MemberRole>,

}

impl Route {
    pub fn url(&self) -> String {
        format!("{}/{}", self.parent_path, self.segments.join("/"))
    }
}


pub fn scoped(
    path: &'static str,
    group: &'static str,
    roles: Option<MemberRole>,
    inner: actix_web::Scope,
) -> actix_web::Scope {

    SCOPE_CTX.with(|ctx| {
        let mut ctx = ctx.borrow_mut();

        let prev = ctx.clone();

        ctx.prefix.push_str(path);
        if !ctx.group.is_empty() {
            ctx.group.push('.');
        }
        ctx.group.push_str(group);

        if let Some(r) = roles {
            ctx.roles = r;
        }

        let scope = actix_web::web::scope(path).service(inner);

        *ctx = prev;

        scope
    })
}

use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static ROUTES: Lazy<Mutex<Vec<Route>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn routes() -> &'static Mutex<Vec<Route>> {
    &ROUTES
}


pub fn register<F, T>(
    key: &'static str,
    method: Method,
    context: &str,
    path: &'static str,
    handler: F,
    roles: MemberRole,
) -> actix_web::Resource
where
    F: actix_web::Handler<T> + Clone + 'static,
    T: actix_web::FromRequest + 'static,
    <F as actix_web::Handler<T>>::Output: actix_web::Responder,
{
    let (segments, effective_roles) = SCOPE_CTX.with(|ctx| {
        let ctx = ctx.borrow();

      
        // build prefix segments safely
        let mut segs: Vec<String> = ctx
            .prefix
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        // normalize incoming path
        if !path.is_empty() && path != "/" {
            segs.push(path.trim_matches('/').to_string());
        }

        // inherit scope role unless explicitly overridden
        let effective_roles = if roles == MemberRole::Public {
            ctx.roles
        } else {
            roles
        };

        (segs, effective_roles)
    });

    let mut table = routes().lock().unwrap();

    // 🚫 prevent duplicates (multi-worker safe)
    let exists = table.iter().any(|r| {
        r.key == key &&
        r.method == method.as_str() &&
        r.segments == segments
    });

    if !exists {
        table.push(Route {
            parent_path: context.to_string(),
            key,
            segments: segments.clone(),
            method: method.as_str(),
            auth: effective_roles != MemberRole::Public,
            roles: vec![effective_roles],
        });

        //log::debug!("Registered route: {} {}", method.as_str(), key);
    }

    actix_web::web::resource(path)
        .route(method.to_route().to(handler))
}


pub fn api_scope(path: &'static str) -> Scope {
    //.service(scoped("api", Some(MemberRole::Public), web::scope(""))
    web::scope(path)
        .service(scoped("/auth", "auth", Some(MemberRole::Public), authentication::scope(vec![path, "auth"])))
        .service(scoped("/users", "users", Some(MemberRole::Member),users_api::scope(vec![path, "users"])))
        .service(scoped("/config","config", Some(MemberRole::Public), config::scope(vec![path, "config"])))
        .service(scoped("/roles", "roles", Some(MemberRole::Member),roles_api::scope(vec![path, "roles"])))
        .service(scoped("/profile","profile", Some(MemberRole::Member), profile::scope(vec![path, "profile"])))
        .service(scoped("/memberships","membership", Some(MemberRole::Member), memberships_api::scope(vec![path, "membership"])))
        .service(scoped("/offers","offers", Some(MemberRole::Member), offers_api::scope(vec![path, "offers"])))
        
        .service(scoped("/ratings", "ratings", Some(MemberRole::Member),ratings_api::scope(vec![path, "ratings"])))
        
        .service(scoped("/upload", "upload", Some(MemberRole::Member),uploads::scope(vec![path, "upload"])))
        .service(scoped("/weekly-answers", "weekly-answers", Some(MemberRole::Member),weekly_answers::scope(vec![path, "weekly-answers"])))
        .service(scoped("/ticket", "ticket", Some(MemberRole::Public),ticket_api::scope(vec![path, "ticket"])))
        .service(scoped("/mail", "mail", Some(MemberRole::Public),mailing_list::scope(vec![path, "mail"])))
        .service(scoped("/celebrate","celebrate", Some(MemberRole::Public), contribution_event::scope(vec![path, "celebrate"])))
        // Twilio integration example
        .service(scoped("/twilio", "twilio", Some(MemberRole::Public),twilio::scope(vec![path, "twilio"])))
        
        .service(
    web::scope("/admin")
            .wrap(AdminMiddleware)
            .service(admin_scope())
                
        )
        

}


pub fn admin_scope() -> Scope {    
    let path= "/api/admin";
    web::scope("")
        .service(scoped("/events", "events", Some(MemberRole::Member), events_api::admin_scope(vec![path, "events"])))
        .service(scoped("/contrib_context", "contrib", None ,   contribution_event::admin_scope(vec![path, "contrib"])))
        .service(scoped("/drafts","drafts", None,  drafts_api::scope(vec![path, "drafts"])))
        .service(scoped("/memberships","memberships", None,  memberships_api::admin_scope(vec![path, "membership"])))
        .service(scoped("/hosts", "hosts", None, hosts::admin_scope( vec![path, "hosts"] )))
        .service(scoped("/users", "users", None, users_api::admin_scope( vec![path, "users"] )))
        .service(scoped("/mail", "mail", Some(MemberRole::Admin),mailing_list::admin_scope(vec![path, "mail"])))

}


// pub fn admin_scope(path: &str) -> Scope {    
//     web::scope(path)
//         .service(scoped("/contrib_context",     contribution_event::admin_scope()))
//         .service(scoped("/drafts", drafts_api::scope()))
//         .service(scoped("/memberships", memberships_api::admin_scope()))
//         .service(hosts::admin_scope("/hosts"))
// }
