use crate::db::DbPool;
use crate::errors::app_error::AppError;
use crate::schema::hosts::dsl::*;
use diesel::prelude::*;

/// DB struct for hosts
#[derive(Queryable)]
pub struct Host {
    pub id: i32,
    pub slug: String,
    pub host_name: String,
    pub display_name: String,
    pub base_url: String,
    pub created_at: chrono::NaiveDateTime,
    pub active: bool,
}

#[derive(Clone)]
pub struct HostsService {
    db_pool: DbPool,
}

impl HostsService {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub fn get_host_by_name(&self, host_name_str: &str) -> Result<Host, AppError> {
        let conn = self.db_pool.get();
        let mut conn = match conn {
            Ok(conn) => conn,
            Err(err) => {
                return Err(AppError::Db(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(err.to_string()),
                )));
            }
        };
        let host_entry = hosts
            .filter(host_name.eq(host_name_str))
            .first::<Host>(&mut conn)
            .optional()
            .map_err(AppError::Db)?;

        // If not found, return a fallback "Unknown" host
        let host_entry = match host_entry {
            Some(host) => host,
            None => Host {
                id: 0,                     // your reserved ID for unknown
                slug: "unknown".into(),
                host_name: "unknown".into(),
                display_name: "Unknown Host".into(),
                base_url: "".into(),
                created_at: chrono::Utc::now().naive_utc(),
                active: false,
            },
        };
        Ok(host_entry)
    }
}
