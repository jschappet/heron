use crate::db::{DbConn, DbPool};
use crate::errors::app_error::AppError;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use handlebars::Handlebars;
use crate::settings::Settings;



#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub hb: Arc<Handlebars<'static>>,
    pub settings: Settings,
    pub rate_limiter: Arc<Mutex<HashMap<String, Instant>>>,
}

impl AppState {
    /// Get a pooled connection from the pool
    pub fn db_conn(&self) -> Result<DbConn, AppError> {
        let conn = self.db_pool.get(); 
        match conn {
            Ok(conn) => Ok(conn),
            Err(err) => {
                
                Err(AppError::User(err.to_string()))
            },
        } 
    }
}
