use actix_web::{HttpResponse, Scope, delete, get, post, web};

use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::models::memberships::*;
use crate::validator::AuthContext;
use crate::routes::register;
use crate::types::method::Method;

// #[post("")]
pub async fn create_membership_api(
    data: web::Data<AppState>,
    new_m: web::Json<NewMembership>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let m = create_membership(&mut conn, &new_m.into_inner())?; 
    Ok(HttpResponse::Ok().json(m))
}

// #[get("")]
pub async fn get_memberships_api(
    data: web::Data<AppState>,
    _user: AuthContext,
) ->  Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let list = get_memberships(&mut conn)?;
    
    Ok(HttpResponse::Ok().json(list))
        
}

// #[get("/user/{id}")]
pub async fn get_memberships_for_user_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let list =  get_memberships_for_user(&mut conn, id.into_inner())?;
    Ok(HttpResponse::Ok().json(list))
}

//#[delete("/{id}")]
pub async fn delete_membership_api(
    data: web::Data<AppState>,
    _auth_context: AuthContext, 
    id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let _ = delete_membership(&mut conn, id.into_inner())?;
    Ok(HttpResponse::Ok().body("Deleted"))
}

pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");

    web::scope("/")
        //.service(create_membership_api)
        // Membership API registrations

// POST / (create membership)
.service(register(
    "create_membership",
    Method::POST,
    &full_path,
    "",
    create_membership_api,
    crate::types::MemberRole::Admin,
))

// GET / (list all memberships)
.service(register(
    "get_memberships",
    Method::GET,
    &full_path,
    "",
    get_memberships_api,
    crate::types::MemberRole::Admin,
))


        
}

pub fn admin_scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");

    web::scope("")
// Membership API registrations

// POST / (create membership)
.service(register(
    "create_membership",
    Method::POST,
    &full_path,
    "",
    create_membership_api,
    crate::types::MemberRole::Admin,
))

// GET /user/{id} (list memberships for a specific user)
.service(register(
    "get_memberships_for_user",
    Method::GET,
    &full_path,
    "user/{id}",
    get_memberships_for_user_api,
    crate::types::MemberRole::Admin,
))

// DELETE /{id} (delete membership)
.service(register(
    "delete_membership",
    Method::DELETE,
    &full_path,
    "{id}",
    delete_membership_api,
    crate::types::MemberRole::Admin,
))

}
// .service(get_memberships_api)
//         .service(get_memberships_for_user_api)
//         .service(create_membership_api)
//         .service(get_memberships_for_user_api)
//         .service(delete_membership_api)