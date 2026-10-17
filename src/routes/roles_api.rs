use actix_web::{HttpResponse, Responder, Scope, web};

use crate::app_state::AppState;
use crate::{validator::{AuthContext, require_role_for_host}, middleware::host::HostContext};
use crate::models::roles::*;
use crate::routes::{register, RoutePath};
use crate::types::method::Method;

// #[post("")]
pub async fn create_role_api(
    _data: web::Data<AppState>,
    _new_role: web::Json<NewRole>,
    _auth: AuthContext,
) -> impl Responder {
    HttpResponse::Forbidden().body("Global role definitions are managed by the platform; use memberships to assign roles")
}

// #[get("")]
pub async fn get_roles_api(data: web::Data<AppState>, auth: AuthContext, host: HostContext) -> impl Responder {
    if require_role_for_host(&auth, host.0.id, &[crate::types::MemberRole::Admin]).is_err() { return HttpResponse::Forbidden().finish(); }
    let mut conn = data.db_pool.get().unwrap();
    match get_roles(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// #[get("/{id}")]
pub async fn get_role_api(data: web::Data<AppState>, id: web::Path<i32>, auth: AuthContext, host: HostContext) -> impl Responder {
    if require_role_for_host(&auth, host.0.id, &[crate::types::MemberRole::Admin]).is_err() { return HttpResponse::Forbidden().finish(); }
    let mut conn = data.db_pool.get().unwrap();
    match get_role(&mut conn, id.into_inner()) {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(_) => HttpResponse::NotFound().body("Role not found"),
    }
}

// #[put("/{id}")]
pub async fn update_role_api(
    _data: web::Data<AppState>,
    _id: web::Path<i32>,
    _updated: web::Json<NewRole>,
    _auth: AuthContext,
) -> impl Responder {
    HttpResponse::Forbidden().body("Global role definitions are managed by the platform; use memberships to assign roles")
}

// #[delete("/{id}")]
pub async fn delete_role_api(_data: web::Data<AppState>, _id: web::Path<i32>, _auth: AuthContext) -> impl Responder {
    HttpResponse::Forbidden().body("Global role definitions are managed by the platform; use memberships to assign roles")
}

pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register(
            "role_create",
            Method::POST,
            path.as_str(),
            "",
            create_role_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "roles_list",
            Method::GET,
            path.as_str(),
            "list",
            get_roles_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "role_get",
            Method::GET,
            path.as_str(),
            "{id}",
            get_role_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "role_update",
            Method::PUT,
            path.as_str(),
            "{id}",
            update_role_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "role_delete",
            Method::DELETE,
            path.as_str(),
            "{id}",
            delete_role_api,
            crate::types::MemberRole::Admin,
        ))
}

// .service(get_roles_api)
//         .service(get_role_api)
//         .service(update_role_api)
//         .service(delete_role_api)
