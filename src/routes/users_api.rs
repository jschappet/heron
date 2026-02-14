use actix_web::{HttpRequest, HttpResponse, Responder, Scope, delete, get, post, put, web};
use serde::Deserialize;
//use crate::crud::{create_user, get_users, get_user, update_user, delete_user};
use crate::AppState;
use crate::errors::app_error::AppError;
use crate::middleware::host_utils::require_host;
use crate::routes::mailing_list::send_password_reset_email;
use crate::models::user_token::{create_user_token, verify_user_token};
use crate::models::users::{
    NewUser, PublicUser, User, create_user, delete_user, get_public_users, get_user, get_user_by_username, get_user_by_username_or_email, get_users, set_password, update_user, update_user_details
};
use crate::schema::hosts::display_name;
use crate::settings::DeployedEnvironment;
use crate::types::TokenPurpose;
use crate::validator::AuthContext;

#[derive(Deserialize)]
pub struct UpdateUserDetailsRequest {
    //pub user_id: i32,
    pub user_details: serde_json::Value,
}

#[post("")]
async fn create_user_api(
    data: web::Data<AppState>,
    new_user: web::Json<NewUser>,
) -> impl Responder {
    let pool = &data.db_pool;
    let mut conn = pool.get().expect("Failed to get DB connection");
    match create_user(&mut conn, &new_user.username, Some(&new_user.email)) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create user"),
    }
}

#[get("/users/page")]
async fn get_users_page(data: web::Data<AppState>) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_users(conn) {
        Ok(user_data) => {
            let html = format!(
                r#"{}"#,
                user_data
                    .into_iter()
                    .map(|user| format!("<li>{}: {}</li>", user.username, user.email))
                    .collect::<Vec<String>>()
                    .join("")
            );
            HttpResponse::Ok().content_type("text/html").body(html)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve tasks"),
    }
}

#[get("")]
async fn get_users_api(data: web::Data<AppState>) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_users(conn) {
        Ok(user_data) => HttpResponse::Ok().json(user_data),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve users"),
    }
}

#[get("/public_profile")]
async fn get_public_users_api(
    data: web::Data<AppState>,
    auth_context: AuthContext,
) -> impl Responder {
    let _user = auth_context; // Currently unused, but can be used for auth checks
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_public_users(conn) {
        Ok(user_list) => {
            let public_users: Vec<PublicUser> =
                user_list.into_iter().map(PublicUser::from).collect();
            // Fileter users based on show_in_directory flag
            let public_users: Vec<PublicUser> = public_users
                .into_iter()
                .filter(|user| user.show_in_directory.unwrap_or(false))
                .collect();
            HttpResponse::Ok().json(public_users)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve user"),
    }
}

#[get("/public_profile/user/{user_id}")]
async fn get_public_user_api(
    data: web::Data<AppState>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_user(conn, user_id.into_inner()) {
        Ok(user) => HttpResponse::Ok().json(PublicUser::from(user)),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve user"),
    }
}

#[get("/{user_id}")]
async fn get_user_api(data: web::Data<AppState>, user_id: web::Path<i32>) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_user(conn, user_id.into_inner()) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve user"),
    }
}

#[put("/{user_id}")]
async fn update_user_api(
    data: web::Data<AppState>,
    user_id: web::Path<i32>,
    updated_user: web::Json<User>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match update_user(
        conn,
        user_id.into_inner(),
        &updated_user.password_hash,
        Some(&updated_user.email),
    ) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update user"),
    }
}

#[delete("/{user_id}")]
async fn delete_user_api(data: web::Data<AppState>, user_id: web::Path<i32>) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    let new_id = user_id.into_inner();
    match delete_user(conn, new_id) {
        Ok(_) => HttpResponse::Ok().body(format!("User {:?} deleted", new_id)),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete user"),
    }
}

#[post("/details")]
async fn update_user_details_api(
    data: web::Data<AppState>,
    req: web::Json<UpdateUserDetailsRequest>,
    auth_context: AuthContext,
) -> impl Responder {
    //let _ = req.user_id == user.id || return HttpResponse::Unauthorized().body("Unauthorized");

    let pool = &data.db_pool;
    let mut conn = pool.get().expect("Failed to get DB connection");

    match update_user_details(&mut conn, auth_context.user_id, req.user_details.clone()) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update user details"),
    }
}

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub username: String,
    
}

//https://dev.revillagesociety.org/api/reset-password/b402059d-d92f-4403-a9c8-8e3baa6b161b
#[get("/reset-password/{token}")]
async fn get_reset_password_page(token: web::Path<String>, 
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let token = token.into_inner();

    match verify_user_token(&mut conn, &token, TokenPurpose::ResetPassword) {
        Ok(user_id) => {
            //users::activate_user(&mut conn, user_id)?;
            log::info!("User account verified for user_id: {}", user_id);

            Ok(HttpResponse::Found()
                .append_header(("Location", "/login/?verified=1"))
                .finish())
        }
        Err(e) => {
            log::warn!("Failed to verify account: {}", e);

            Ok(HttpResponse::Found()
                .append_header(("Location", "/login/?error=invalid_or_expired_token"))
                .finish())
        }
    }

}


#[post("/reset-password")]
async fn reset_password_token(
    data: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;


    let incoming_host = require_host(&req).await.unwrap(); // safe because fallback exists
    let base_url = incoming_host.base_url.as_str();
    let host_display_name = incoming_host.display_name.as_str();
    
    let user = get_user_by_username_or_email(&mut conn, &payload.username)
        .map_err(|_| AppError::User("User not found".into()))?;

    if !user.is_active {
        return Err(AppError::User("Account not verified".into()));
    }

    let token = create_user_token(&mut conn, user.id, TokenPurpose::ResetPassword, 15)?;
    // Send email with token
    if let Err(e) = send_password_reset_email(
        &user.email,
        &user.username,
        &token,
        base_url,
        host_display_name,
        &data.settings,
    ) {
        if data.settings.environment == DeployedEnvironment::Development {
            log::warn!("DEV: email skipped: {}", e);
            log::info!("DEV reset token: {}", token);
            // Keep going in dev mode
        } else {
            log::error!("Email send failed: {}", e);
            return Err(AppError::Internal(
                "Failed to send password reset email".into(),
            ))
        }
    } 


    Ok(HttpResponse::Ok().finish())
}

#[post("/set-password")]
async fn set_password_new(
    data: web::Data<AppState>,
    payload: web::Json<SetPasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;

    let user = get_user_by_username(&mut conn, &payload.username)
        .map_err(|_| AppError::User("User not found".into()))?;

    if !user.is_active {
        return Err(AppError::User("Account not verified".into()));
    }

    set_password(&mut conn, user.id, &payload.password)?;

    Ok(HttpResponse::Ok().finish())
}


pub fn scope() -> Scope {
    web::scope("")
        .service(create_user_api)
        .service(get_users_api)
        .service(get_user_api)
        .service(update_user_api)
        .service(delete_user_api)
        .service(get_users_page)
        .service(update_user_details_api)
        .service(get_public_user_api)
        .service(get_public_users_api)
        .service(set_password_new)
        .service(reset_password_token)
        .service(get_reset_password_page)
}

pub fn admin_scope() -> Scope {
    web::scope("")
        .service(create_user_api)
        .service(delete_user_api)
        .service(update_user_details_api)
        .service(set_password_new)
        .service(reset_password_token)
        .service(get_reset_password_page)
}