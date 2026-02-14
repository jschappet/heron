use super::app_error::ErrorResponse;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    NotAuthenticated,
    Forbidden(&'static str),
    Internal(String),
    InvalidToken(&'static str),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::NotAuthenticated => write!(f, "Not authenticated"),
            AuthError::Forbidden(msg) => write!(f, "{msg}"),
            AuthError::Internal(msg) => write!(f, "Internal Error {}", msg),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
        }
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::NotAuthenticated => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: 401,
                    error_type: "NotAuthenticated",
                    message: "Authentication required".to_string(),
                })
            }

            AuthError::Forbidden(msg) => {
                HttpResponse::Forbidden().json(ErrorResponse {
                    code: 403,
                    error_type: "Forbidden",
                    message: msg.to_string(),
                })
            }

            AuthError::InvalidToken(msg) => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: 401,
                    error_type: "InvalidToken",
                    message: msg.to_string(),
                })
            }

            AuthError::Internal(e) => {
                let error_id = uuid::Uuid::new_v4();
                log::error!("Auth Error Internal error [{}]: {}", error_id, e);

                HttpResponse::InternalServerError().json(ErrorResponse {
                    code: 500,
                    error_type: "InternalError",
                    message: format!("Internal server error: {}", error_id),
                })
            }
        }
    }
}
