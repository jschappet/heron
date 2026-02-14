use actix_web::{App, HttpResponse, ResponseError};
use bcrypt::BcryptError;
use diesel::r2d2::{self, Error as R2D2Error};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::errors::auth_error::AuthError;

/// Standard API error JSON
#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub error_type: &'static str,
    pub message: String,
    // optional: request_id or correlation_id for tracing
    // request_id: Option<String>,
}

/// Centralized app error type
#[derive(Debug)]
pub enum AppError {
    User(String),
    Auth(AuthError),
    Db(diesel::result::Error),
    
    R2D2(R2D2Error),
    Internal(String),
    BcryptError(BcryptError),
    NotFound(String), 
    BadRequest(String),
    Unauthorized,

}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::User(e) => write!(f, "{e}"),
            AppError::Auth(e) => write!(f, "{e}"),
            AppError::Db(e) => write!(f, "Database error: {}", e),
            AppError::R2D2(e) => write!(f, "Connection pool error: {}", e),
            AppError::Internal(e) => write!(f, "{e}"),
            AppError::BcryptError(e) => write!(f, "Bcrypt error: {}", e),
            AppError::NotFound(e) => write!(f, "Not found: {}", e), // ← display message
            AppError::BadRequest(e) => write!(f, "Not found: {}", e), // ← display message
            AppError::Unauthorized => write!(f, "Unauthorized"), // ← display message

        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound(msg) => {
                let resp = ErrorResponse {
                    code: 404,
                    error_type: "NotFound",
                    message: msg.clone(),
                };
                log::warn!("NotFound: {}", msg);
                HttpResponse::NotFound().json(resp)
            }
            AppError::Unauthorized => {
                let resp = ErrorResponse {
                    code: 400,
                    error_type: "UserError",
                    message: String::from("Unauthorized"),
                };
                
                HttpResponse::InternalServerError().json(resp)
            }
            AppError::Auth(e) => {
                // Delegate auth errors
                e.error_response()
            }
            AppError::User(e) => {
                let resp = ErrorResponse {
                    code: 500,
                    error_type: "UserError",
                    message: e.clone(),
                };
                log::error!("UserError: {}", e);
                HttpResponse::InternalServerError().json(resp)
            }
            AppError::BadRequest(e) => {
                let resp = ErrorResponse {
                    code: 500,
                    error_type: "UserError",
                    message: e.clone(),
                };
                log::error!("BadRequest: {}", e);
                HttpResponse::InternalServerError().json(resp)
            }
            AppError::Db(e) => {
                let error_id = uuid::Uuid::new_v4();
                log::error!("DB Error [{}]: {:?}", error_id, e);
                let resp = ErrorResponse {
                    code: 500,
                    error_type: "DbError",
                    message: format!("Internal server error: {}", error_id),
                };
                HttpResponse::InternalServerError().json(resp)
            }
            AppError::R2D2(e) => {
                let error_id = uuid::Uuid::new_v4();
                log::error!("Connection pool error [{}]: {:?}", error_id, e);
                let resp = ErrorResponse {
                    code: 500,
                    error_type: "R2D2Error",
                    message: format!("Internal server error: {}", error_id),
                };
                HttpResponse::InternalServerError().json(resp)
            }
            AppError::Internal(e) => {
                let error_id = uuid::Uuid::new_v4();
                log::error!("Internal error [{}]: {}", error_id, e);
                let resp = ErrorResponse {
                    code: 500,
                    error_type: "InternalError",
                    message: format!("Internal server error: {}", error_id),
                };
                HttpResponse::InternalServerError().json(resp)
            },
            AppError::BcryptError(e) => {
                let error_id = uuid::Uuid::new_v4();
                log::error!("Bcrypt error [{}]: {:?}", error_id, e);
                let resp = ErrorResponse {
                    code: 500,
                    error_type: "BcryptError",
                    message: format!("Internal server error: {}", error_id),
                };
                HttpResponse::InternalServerError().json(resp)
            },
        }
    }
}

// Allow `?` operator conversions
impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        AppError::Auth(err)
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::Db(err)
    }
}

impl From<R2D2Error> for AppError {
    fn from(err: R2D2Error) -> Self {
        AppError::R2D2(err)
    }
}



impl From<BcryptError> for AppError {
    fn from(err: BcryptError) -> Self {
        AppError::BcryptError(err)
    }
}
