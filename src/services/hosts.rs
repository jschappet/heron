use crate::db::{DbConn, DbPool};
use crate::errors::app_error::AppError;
use crate::schema::hosts::dsl::*;
use actix_web::App;
use diesel::prelude::*;
use serde::Serialize;
use crate::schema::hosts::dsl::hosts as hosts_dsl;

/// DB struct for hosts
#[derive(Queryable, Serialize)]
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
pub struct HostDomain {
    service: HostsService,
}

impl HostDomain {
    pub fn new(pool: DbPool) -> Self {
        Self {
            service: HostsService::new(pool),
        }
    }

    pub fn get_host_by_name(
        &self,
        host_name_str: &str,
    ) -> Result<Host, AppError> {
        self.service.get_host_by_name(host_name_str)
    }

    pub fn get_host_list(&self) -> Result<Vec<Host>, AppError> {
        self.service.get_host_list()
    }

}

#[derive(Clone)]
pub struct HostsService {
    db_pool: DbPool,
}

impl HostsService {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub fn db_conn(&self) -> Result<DbConn, AppError> {
        self.db_pool
            .get()
            .map_err(|err| AppError::User(err.to_string()))
    }

    pub fn get_host_list(&self) -> Result<Vec<Host>, AppError> {
        let mut conn = self.db_conn()?;
        match hosts_dsl.load::<Host>(&mut conn) {
            Ok(hosts_list) => Ok(hosts_list),
            Err(e) => Err(AppError::Db(e)),
        }
            
    }

    pub fn get_host_by_name(&self, host_name_str: &str) -> Result<Host, AppError> {
        let mut conn = self.db_conn()?;
    
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
