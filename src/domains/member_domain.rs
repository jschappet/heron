use serde::Serialize;
use serde_json::{Value, json};

use crate::db::DbPool;
use crate::errors::app_error::AppError;
use crate::services::member_content_service::{MemberContent, MemberContentService, MemberFlows};

#[derive(Clone)]
pub struct MemberDomain {
    pool: DbPool,
}


impl MemberDomain {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<crate::db::DbConn, AppError> {
        self.pool.get().map_err(|e| AppError::User(e.to_string()))
    }

    pub fn member_content(&self, host_id: i32, member_id: i32) -> Result<MemberContent, AppError> {
        let mut conn = self.conn()?;
        MemberContentService::member_content(&mut conn, host_id, member_id)
    }

     pub fn member_flows(&self, host_id: i32, member_id: i32) -> Result<MemberFlows, AppError> {
        let mut conn = self.conn()?;
        MemberContentService::member_flows(&mut conn, host_id, member_id)
    }
}