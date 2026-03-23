use crate::domains::ledger_domain::LedgerDomain;
use crate::errors::app_error::AppError;
use crate::models::drafts::{Draft, NewDraft};
use crate::services::draft_service::DraftListItem;
use crate::services::ledger_service::{self, LedgerService};
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
use serde_json::json;

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

    pub fn create_draft(&self, new: &NewDraft) -> Result<Draft, AppError> {
        let mut conn = self.conn()?;

        let draft = DraftService::create_draft(&mut conn, &new)?;

        let timestamp = chrono::Utc::now().naive_utc();
        let ledger_domain = LedgerDomain::new(self.pool.clone());

        let entity_user_id = ledger_domain
            .resolve_or_create_entity(new.submitted_by.unwrap_or(0), new.host_id)
            .unwrap();

        let from_id = entity_user_id.clone();
        let to_id = ledger_domain
            .resolve_entity("IdeaBank", new.host_id)
            .unwrap();

        let new_flow = crate::models::flow_events::NewFlowEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp,
            from_entity: from_id,
            to_entity: to_id,
            host_id: new.host_id,
            resource_type: "document_submitted".to_string(),
            quantity_value: 1.0,
            quantity_unit: "count".to_string(),
            notes: Some(json!({ "doc_type": draft.doc_type, "draft_id": draft.id }).to_string()),
            recorded_at: chrono::Utc::now().naive_utc(),
            details: JsonField::default(),
            created_by: entity_user_id,
        };

        if let Err(e) = ledger_domain.record_flow(new_flow) {
            log::error!("Failed to save FlowEvent {:?}", e);
        }

        Ok(draft)
    }
}
