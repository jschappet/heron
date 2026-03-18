use crate::errors::app_error::AppError;
use crate::services::draft_service::DraftListItem;
use crate::services::member_content_service::MemberContent;
use crate::{db::DbPool, services::draft_service::DraftService};

use crate::{
    routes::drafts_api::{self, DraftQuery},
    schema::drafts,
    types::{DocType, DraftStatus, JsonField},
};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DraftDomain {
    pool: DbPool,
}

impl DraftDomain {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<crate::db::DbConn, AppError> {
        self.pool.get().map_err(|e| AppError::User(e.to_string()))
    }

    pub fn get_draft_list_for_user(
        &self,
        host_id: i32,
        member_id: i32,
    ) -> Result<MemberContent, AppError> {
        let mut conn = self.conn()?;
        let list = DraftService::get_draft_list_for_user(&mut conn, host_id, member_id)?;
        Ok(MemberContent { draft_list: list })
    }
}
