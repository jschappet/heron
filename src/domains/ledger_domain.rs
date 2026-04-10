use std::collections::HashMap;

use diesel::dsl::json;
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::db::DbPool;
use crate::errors::app_error::AppError;

use crate::models::entities::{Entity, NewEntity};
use crate::models::flow_events::{FlowEvent, NewFlowEvent};

//use crate::models::{Entity, FlowEvent, NewEntity, NewFlowEvent};
use crate::services::ledger_service::{EntityRef, LedgerEventRow, LedgerService};
use crate::types::flow_query::FlowQuery;
use crate::types::{Audience, ConfigHash, JsonField};

#[derive(Serialize)]
pub struct EntityFlows {
    pub entity_id: String,
    pub inflows: Vec<LedgerEventRow>,
    pub outflows: Vec<LedgerEventRow>,
    pub entities: Vec<EntityRef>,
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

    pub fn get_effort_contexts(
        &self,
        host: i32,
        audience: Audience,
    ) -> Result<Vec<ConfigHash>, AppError> {
        let mut conn = self.conn()?;
        LedgerService::get_effort_contexts(&mut conn, host, audience)
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
        LedgerService::get_user_entity_id(&mut conn, host, user)
            .map_err(|e| AppError::User(e.to_string()))
    }

    // FLOW EVENTS

    pub fn record_flow(&self, new: NewFlowEvent) -> Result<FlowEvent, AppError> {
        let mut conn = self.conn()?;
        LedgerService::create_flow_event(&mut conn, new).map_err(|e| AppError::User(e.to_string()))
    }

    pub fn get_flow_events(
        &self,
        host: i32,
    ) -> Result<(Vec<LedgerEventRow>, Vec<EntityRef>, Vec<String>), AppError> {
        let mut conn = self.conn()?;
        LedgerService::get_flow_events(&mut conn, FlowQuery::new(host).both())
            .map_err(|e| AppError::User(e.to_string()))
    }

    pub fn get_flows_for_entity(
        &self,
        host: i32,
        entity_id: &str,
    ) -> Result<EntityFlows, AppError> {
        let entity_id =
            Uuid::parse_str(entity_id).map_err(|e| AppError::User(format!("Bad UUID: {}", e)))?;

        let mut conn = self.conn()?;
        let (inflows, inflow_entities, _) =
            LedgerService::get_flow_events(&mut conn, FlowQuery::new(host).to().entity(entity_id))?;
        let (outflows, outflow_entities, _) = LedgerService::get_flow_events(
            &mut conn,
            FlowQuery::new(host).from().entity(entity_id),
        )?;

        // Merge entities, deduplicating by id
        let mut entity_map: HashMap<String, EntityRef> = HashMap::new();

        for e in inflow_entities
            .into_iter()
            .chain(outflow_entities.into_iter())
        {
            entity_map.entry(e.id.clone()).or_insert(e);
        }

        let entities: Vec<EntityRef> = entity_map.into_values().collect();

        Ok(EntityFlows {
            entity_id: entity_id.to_string(),
            inflows,
            outflows,
            entities,
        })
    }

    pub fn get_ledger_summary(&self, flow_query: FlowQuery) -> Result<Value, AppError> {
        let mut conn = self.conn()?;
        let events = LedgerService::get_flow_events(&mut conn, flow_query)?;
        Ok(json!(
            { "total_events": events.0.len(),
            "ledger": events.0,
            "entities": events.1 ,
            "resources": events.2
        }))
    }

    pub fn resolve_or_create_entity(&self, input: i32, host: i32) -> Result<String, AppError> {
        let mut conn = self.conn()?;
        let id = LedgerService::get_user_entity_id(&mut conn, host, input).unwrap();
        Ok(id)
    }

    pub fn merge_entities(
        &self,
        from_entity: &str,
        to_entity: &str,
        host: i32,
        user: i32,
    ) -> Result<(), AppError> {
        let mut conn = self.conn()?;

        LedgerService::merge_entities(&mut conn, from_entity, to_entity, host, user)
            .map_err(|e| AppError::User(e.to_string()))?;

        Ok(())
    }
    pub fn find_or_create_entity(&self, input: &str, host: i32) -> Result<String, AppError> {
        let mut conn = self.conn()?;

        if let Ok(entity) = LedgerService::find_entity_by_name(&mut conn, input, host) {
            return Ok(entity.id);
        }

        // Only create if it's truly not found
        match LedgerService::find_entity_by_name(&mut conn, input, host) {
            Err(AppError::NotFound(_)) => {}
            Err(e) => return Err(e),
            _ => {}
        }

        let id = Uuid::new_v4().to_string();

        let new_entity = NewEntity {
            id: id.clone(),
            name: input.to_string(),
            host_id: host,
            entity_type: "person_email".to_string(),
            created_by: "System".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
            details: JsonField::default(),
            entity_type_id: None,
            canonical_entity_id: Some(id.clone()),
        };

        let result = LedgerService::create_entity(&mut conn, new_entity)?;
        Ok(result.id)
    }

    pub fn _off_find_or_create_entity(&self, input: &str, host: i32) -> Result<String, AppError> {
        let mut conn = self.conn()?;
        match LedgerService::find_entity_by_name(&mut conn, input, host) {
            Ok(entity) => Ok(entity.id),
            Err(_) => {
                let new_uuid = Uuid::new_v4().to_string();
                let new_entity = NewEntity {
                    id: new_uuid.clone(),
                    name: input.to_string(),
                    host_id: host,
                    entity_type: "person_email".to_string(),
                    created_by: "System".to_string(),
                    created_at: chrono::Utc::now().naive_utc(),
                    details: JsonField::default(),
                    entity_type_id: None,
                    canonical_entity_id: Some(new_uuid.clone()),
                };
                let result = LedgerService::create_entity(&mut conn, new_entity)
                    .map_err(|e| AppError::User(e.to_string()))?;
                Ok(result.id)
            }
        }
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
