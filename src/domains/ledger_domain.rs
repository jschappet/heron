use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::db::DbPool;
use crate::errors::app_error::AppError;

use crate::models::entities::{Entity, NewEntity};
use crate::models::flow_events::{FlowEvent, NewFlowEvent};

//use crate::models::{Entity, FlowEvent, NewEntity, NewFlowEvent};
use crate::services::ledger_service::{LedgerEventDto, LedgerService};

#[derive(Serialize)]
pub struct EntityFlows {
    pub entity_id: String,
    pub inflows: Vec<LedgerEventDto>,
    pub outflows: Vec<LedgerEventDto>,
}

#[derive(Clone)]
pub struct LedgerDomain {
    pool: DbPool,
}

impl LedgerDomain {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<crate::db::DbConn, AppError> {
        self.pool.get().map_err(|e| AppError::User(e.to_string()))
    }

    // ENTITY CRUD

    pub fn create_entity(&self, new: NewEntity) -> Result<Entity, AppError> {
        let mut conn = self.conn()?;
        LedgerService::create_entity(&mut conn, new).map_err(|e| AppError::User(e.to_string()))
    }


    pub fn save_all_entities(&self, events: Vec<NewEntity>) -> Result<String, AppError> {
        let mut conn = self.conn()?;
        let r = LedgerService::save_all_entities(&mut conn, events);
        Ok(r?)
    }

    pub fn save_all_flow_events(&self, events: Vec<NewFlowEvent>) -> Result<String, AppError> {
        let mut conn = self.conn()?;
        let r = LedgerService::save_all_flow_events(&mut conn, events);
        Ok(r?)
    }

    pub fn get_entity(&self, id: &str) -> Result<Entity, AppError> {
        let mut conn = self.conn()?;
        LedgerService::get_entity(&mut conn, id).map_err(|e| AppError::User(e.to_string()))
    }

    pub fn get_all_entities(&self, host_id: i32) -> Result<Vec<Entity>, AppError> {
        let mut conn = self.conn()?;
        LedgerService::get_entities(&mut conn, host_id).map_err(|e| AppError::User(e.to_string()))
    }

    pub fn get_user_entity_id(&self, host: i32, user: i32) -> Result<String, AppError> {
        let mut conn = self.conn()?;
        LedgerService::get_user_entity_id(&mut conn, host, user).map_err(|e| AppError::User(e.to_string()))
    }

    // FLOW EVENTS

    pub fn record_flow(&self, new: NewFlowEvent) -> Result<FlowEvent, AppError> {
        let mut conn = self.conn()?;
        LedgerService::create_flow_event(&mut conn, new).map_err(|e| AppError::User(e.to_string()))
    }

    pub fn get_flow_events(&self, host: i32) -> Result<Vec<LedgerEventDto>, AppError> {
        let mut conn = self.conn()?;
        LedgerService::get_flow_events(&mut conn, host, None)
            .map_err(|e| AppError::User(e.to_string()))
    }

    pub fn get_flows_for_entity(
        &self,
        host: i32,
        entity_id: &str,
    ) -> Result<EntityFlows, AppError> {
        let mut conn = self.conn()?;
        let events = LedgerService::get_flow_events(&mut conn, host, None)?;

        let inflows: Vec<_> = events
            .iter()
            .filter(|f| f.to.id == entity_id.to_string())
            .cloned()
            .collect();

        let outflows: Vec<_> = events
            .iter()
            .filter(|f| f.from.id == entity_id.to_string())
            .cloned()
            .collect();
        Ok(EntityFlows {
            entity_id: entity_id.to_string(),
            inflows,
            outflows,
        })
    }

    pub fn get_ledger_summary(&self, host: i32) -> Result<Value, AppError> {
        let mut conn = self.conn()?;
        let events = LedgerService::get_flow_events(&mut conn, host, None)?;
        Ok(json!({ "total_events": events.len(), "ledger": events }))
    }

    pub fn resolve_or_create_entity(&self, input: i32, host: i32) -> Result<String, AppError> {
        let mut conn = self.conn()?;
        let id = LedgerService::get_user_entity_id(&mut conn, host, input).unwrap();
        Ok(id)
    }

    // pub fn resolve_or_create_entity_email(&self, input: String, host: i32) -> Result<String, AppError> {
    //     let mut conn = self.conn()?;
    //     let id = LedgerService::get_user_entity_string(&mut conn, host, input).unwrap();
    //     Ok(id)
    // }

    pub fn resolve_entity(&self, input: &str, host: i32) -> Result<String, AppError> {
        let mut conn = self.conn()?;
        if uuid::Uuid::parse_str(input).is_ok() {
            return Ok(input.to_string());
        }
        let entity = LedgerService::find_entity_by_name(&mut conn, input, host)?;

        Ok(entity.id)
    }
}
