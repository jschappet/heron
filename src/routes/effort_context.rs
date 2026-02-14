
use actix_web::{Scope, get, post, put, delete, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, errors::app_error::AppError};
//, types::AdminContext

pub fn admin_scope() -> Scope {
    web::scope("/effort_context")
      //  .service(list_admin)
     //   .service(create)
      //  .service(update)
       // .service(deactivate)
       // .service(reassign_work)
}

/* 
#[get("")]
pub async fn list_admin(
    admin: AdminContext,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let results = state
        .effort_context_service
        .list_admin(&admin)
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
*/
/* 
#[derive(Deserialize)]
pub struct CreateEffortContext {
    pub label: String,
}

#[post("")]
pub async fn create(
    admin: AdminContext,
    state: web::Data<AppState>,
    payload: web::Json<CreateEffortContext>,
) -> Result<HttpResponse, AppError> {
    let created = state
        .effort_context_service
        .create_by_admin(&admin, payload.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(created))
}

#[derive(Deserialize)]
pub struct UpdateEffortContext {
    pub label: Option<String>,
    pub active: Option<bool>,
}

#[put("/{id}")]
pub async fn update(
    admin: AdminContext,
    state: web::Data<AppState>,
    path: web::Path<i32>,
    payload: web::Json<UpdateEffortContext>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let updated = state
        .effort_context_service
        .update(&admin, id, payload.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/{id}")]
pub async fn deactivate(
    admin: AdminContext,
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    state
        .effort_context_service
        .deactivate(&admin, id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct ReassignWorkRequest {
    pub to_context_id: i32,
}

#[post("/{id}/reassign-work")]
pub async fn reassign_work(
    admin: AdminContext,
    state: web::Data<AppState>,
    path: web::Path<i32>,
    payload: web::Json<ReassignWorkRequest>,
) -> Result<HttpResponse, AppError> {
    let from_id = path.into_inner();

    state
        .effort_context_service
        .reassign_work(
            &admin,
            from_id,
            payload.to_context_id,
        )
        .await?;

    Ok(HttpResponse::Ok().finish())
}
*/