use crate::{
    app_state::AppState,
    errors::app_error::AppError,
    models::{
        offers::get_user_offers,
        roles::load_roles,
        users::{PublicUser, get_user},
    },
    validator::AuthContext,
};
use actix_session::Session;
use actix_web::{HttpResponse, Scope, get, web};

use serde_json::json;

#[get("")]
pub async fn profile_json(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    // Try to get the user_id from the session
    let user_id = match session.get::<i32>("user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            // Anonymous user
            return Ok(HttpResponse::Ok().json(json!({
                "authenticated": false,
                "user": null,
                "roles": []
            })));
        }
        Err(err) => {
            // Session read failed: log and return structured internal error
            let error_id = uuid::Uuid::new_v4();
            log::error!("Session read failed [{}]: {}", error_id, err);
            return Err(AppError::Internal(format!(
                "Internal server error: {}",
                error_id
            )));
        }
    };

    // Get DB connection
    let mut conn = data.db_conn()?;

    // Load user
    let user = get_user(&mut conn, user_id)?;
    if !user.is_active {
        return Ok(HttpResponse::Ok().json(json!({
                "authenticated": false,
                "user": null,
                "roles": []
            })));
    }
    // Load roles
    let roles = load_roles(&mut conn, user_id)?;

    // Return structured JSON response
    Ok(HttpResponse::Ok().json(json!({
        "authenticated": true,
        "user": PublicUser::from(user),
        "roles": roles
    })))
}

#[get("/offers")]
pub async fn profile_offers_json(
    data: web::Data<AppState>,
    auth_context: AuthContext,
) -> Result<HttpResponse, AppError> {
    // TODO: Replace this with real DB lookups
    let mut conn = data.db_conn()?;

    let offers = get_user_offers(&mut conn, auth_context.user_id)?;

    Ok(HttpResponse::Ok().json(offers))
}

#[get("/completed")]
pub async fn profile_completed_json(
    data: web::Data<AppState>,
    auth_context: AuthContext,
) -> Result<HttpResponse, AppError> {
    // TODO: Replace this with real DB lookups
    let mut conn = data.db_conn()?;
    let user = get_user(&mut conn, auth_context.user_id)?;
    let mock_response = serde_json::json!({"current_user": user,
      "completed_offers": [
        {
          "id": "offer_2",
          "title": "Firewood Stack",
          "status": "completed",
          "offer": "Wheelbarrow of cedar firewood",
          "request": "Help stacking remaining pile",
          "photos": ["/reciprocity/firewood.jpg"],
          "needs_review": true
        }

      ]
        }
    );
    Ok(HttpResponse::Ok().json(mock_response))
}


pub fn scope() -> Scope {
    web::scope("")
        .service(profile_json)
        .service(profile_offers_json)
        .service(profile_completed_json)
}