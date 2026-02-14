use crate::db::{DbConn, DbPool};
use crate::errors::app_error::AppError;

use std::sync::Arc;
use handlebars::Handlebars;
use crate::settings::Settings;



#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub hb: Arc<Handlebars<'static>>,
    pub settings: Settings,
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
