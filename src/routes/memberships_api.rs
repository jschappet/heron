use actix_web::{HttpResponse, Scope, delete, get, post, web};

use crate::AppState;
use crate::errors::app_error::AppError;
use crate::models::memberships::*;
use crate::validator::AuthContext;

#[post("")]
pub async fn create_membership_api(
    data: web::Data<AppState>,
    new_m: web::Json<NewMembership>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let m = create_membership(&mut conn, &new_m.into_inner())?; 
    Ok(HttpResponse::Ok().json(m))
}

#[get("")]
pub async fn get_memberships_api(
    data: web::Data<AppState>,
    _user: AuthContext,
) ->  Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let list = get_memberships(&mut conn)?;
    
    Ok(HttpResponse::Ok().json(list))
        
}

#[get("/user/{id}")]
pub async fn get_memberships_for_user_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let list =  get_memberships_for_user(&mut conn, id.into_inner())?;
    Ok(HttpResponse::Ok().json(list))
}

#[delete("/{id}")]
pub async fn delete_membership_api(
    data: web::Data<AppState>,
    _auth_context: AuthContext, 
    id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let _ = delete_membership(&mut conn, id.into_inner())?;
    Ok(HttpResponse::Ok().body("Deleted"))
}

pub fn scope() -> Scope {
    web::scope("/")
        .service(create_membership_api)
        .service(get_memberships_api)
        .service(get_memberships_for_user_api)
        .service(delete_membership_api)
}
