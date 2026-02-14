use actix_web::{HttpResponse, Responder, Scope, delete, get, post, put, web};

use crate::AppState;
use crate::models::roles::*;

#[post("")]
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

#[get("")]
pub async fn get_roles_api(data: web::Data<AppState>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_roles(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/{id}")]
pub async fn get_role_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_role(&mut conn, id.into_inner()) {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(_) => HttpResponse::NotFound().body("Role not found"),
    }
}

#[put("/{id}")]
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

#[delete("/{id}")]
pub async fn delete_role_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match delete_role(&mut conn, id.into_inner()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}



pub fn scope() -> Scope {
    web::scope("").service(create_role_api)
        .service(get_roles_api)
        .service(get_role_api)
        .service(update_role_api)
        .service(delete_role_api)
}