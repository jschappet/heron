use actix_web::{HttpResponse, Responder, Scope, delete, get, post, put, web};

use crate::app_state::AppState;
use crate::models::roles::*;
use crate::routes::register;
use crate::types::method::Method;

// #[post("")]
pub async fn create_role_api(
    data: web::Data<AppState>,
    new_role: web::Json<NewRole>,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match create_role(&mut conn, &new_role.into_inner()) {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// #[get("")]
pub async fn get_roles_api(data: web::Data<AppState>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_roles(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// #[get("/{id}")]
pub async fn get_role_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_role(&mut conn, id.into_inner()) {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(_) => HttpResponse::NotFound().body("Role not found"),
    }
}

// #[put("/{id}")]
pub async fn update_role_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    updated: web::Json<NewRole>,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match update_role(&mut conn, id.into_inner(), &updated.into_inner()) {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// #[delete("/{id}")]
pub async fn delete_role_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match delete_role(&mut conn, id.into_inner()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        .service(register(
            "role_create",
            Method::POST,
            &full_path,
            "",
            create_role_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "roles_list",
            Method::GET,
            &full_path,
            "list",
            get_roles_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "role_get",
            Method::GET,
            &full_path,
            "{id}",
            get_role_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "role_update",
            Method::PUT,
            &full_path,
            "{id}",
            update_role_api,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "role_delete",
            Method::DELETE,
            &full_path,
            "{id}",
            delete_role_api,
            crate::types::MemberRole::Admin,
        ))
}

// .service(get_roles_api)
//         .service(get_role_api)
//         .service(update_role_api)
//         .service(delete_role_api)
