use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Scope, get, post, web};
use serde_json::json;

use crate::errors::app_error::AppError;
use crate::errors::auth_error::AuthError;
use crate::middleware::host_utils::require_host;
use crate::models::user_token::verify_user_token;
use crate::schema::hosts::display_name;
use serde::Deserialize;
//use crate::models::offers::*;
//use crate::routes::offers_api;
use crate::app_state::AppState;
use crate::models::users::{self};
use crate::types::TokenPurpose;

#[derive(Deserialize)]
struct RegisterData {
    username: String,
    email: String,
    //password: String,
}

use actix_web::web::Either;

#[post("/register")]
async fn register_new_user(
    data: web::Data<AppState>,
    req: HttpRequest,
    payload: Either<web::Json<RegisterData>, web::Form<RegisterData>>,
) -> Result<HttpResponse, AppError> {

    let incoming_host = require_host(&req).await.unwrap(); // safe because fallback exists
    let base_url = incoming_host.base_url.as_str();
    let site_name = incoming_host.display_name.as_str(); 

    let mut conn = data.db_conn()?;

    let form = match payload {
        Either::Left(json) => json.into_inner(),
        Either::Right(form) => form.into_inner(),
    };

    log::info!("Registering new user: {}", form.username);
    // Check if email or username exists

    users::register_user(
        &mut conn,
        &form.username,
        &form.email,
        base_url, 
        site_name, 
        &data.settings.clone(),
    )?;

    Ok(HttpResponse::Ok().body("User registered successfully"))
}

#[get("/token/{token}")]
async fn verify_account(
    data: web::Data<AppState>,
    token: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let token = token.into_inner();

    match verify_user_token(&mut conn, &token, TokenPurpose::VerifyAccount) {
        Ok(user_id) => {
            users::activate_user(&mut conn, user_id)?;
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

#[post("/register1")]
#[cfg(debug_assertions)]
async fn register_new_user_off(
    data: web::Data<AppState>,
    form: web::Json<RegisterData>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    log::info!("Registering new user: {}", form.username);
    // Check if email or username exists
    if users::get_user_by_username(&mut conn, &form.username.clone()).is_ok() {
        return Err(AppError::User("Username already exists".to_string()));
    }

    if users::get_user_by_email(&mut conn, form.email.clone()).is_ok() {
        return Err(AppError::User("Email already exists".to_string()));
    }

    // Create user
    match users::create_user(&mut conn, &form.username, Some(&form.email)) {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(_) => Err(AppError::User("Failed to create user".to_string())),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct LoginData {
    username: String,
    password: String,
}

#[post("/login")]
async fn login(
    session: Session,
    form: web::Form<LoginData>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let form = form.into_inner();

    match users::authenticate_user(&mut conn, form.username, form.password) {
        Ok(user) => {
            if !user.is_active {
                log::warn!("User {} is not active", user.id);
                return Err(AppError::Auth(AuthError::Forbidden("Inactive user")));
            }
            // store user id in session
            log::info!("User {} logged in {}", user.username, user.id);
            session
                .insert("user_id", user.id)
                .expect("Failed to insert session");
            Ok(HttpResponse::Ok().json(json!({
                "message": format!("Welcome, {}!", user.username)
            })))
            //Ok(HttpResponse::Ok().body(format!("Welcome, {}!", user.username)))
        }
        Err(_) => Err(AppError::Auth(AuthError::Forbidden("Invalid credentials"))),
    }
}

#[post("/logout")]
async fn logout(session: Session) -> Result<HttpResponse, AppError> {
    session.remove("user_id");
    session.purge();

    Ok(HttpResponse::Ok().body("Logged out"))
}



/* =========================
   Public Scope
   ========================= */

pub fn scope() -> Scope {
    web::scope("")
        .service(register_new_user)
        .service(verify_account)
        .service(login)
        .service(logout)
}