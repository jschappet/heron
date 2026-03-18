use serde::{Deserialize, Serialize};
use uuid::Uuid;
  use std::collections::HashMap;

use crate::{db::DbConn, errors::app_error::AppError,
     models::{drafts::Draft, entities::Entity, flow_events::FlowEvent}, routes::member, 
services::{draft_service::{DraftListItem, DraftService}, ledger_service::{ EntityRef, LedgerEventRow, LedgerService}}, types::flow_query::FlowQuery};

#[derive(Serialize, Deserialize)]
pub struct MemberContent {
    pub draft_list: Vec<DraftListItem>,
}


#[derive(Serialize)]
pub struct MemberFlows {
    pub member_uuid: String,
    pub inflows: Vec<LedgerEventRow>,
    pub outflows: Vec<LedgerEventRow>,
    pub entities: Vec<EntityRef>,
}

pub struct MemberContentService;
impl MemberContentService {
    pub fn member_content(conn: &mut DbConn, host_id: i32, member_id: i32) -> Result<MemberContent, AppError> {
        let results = DraftService::get_draft_list_for_user(conn, host_id, member_id);
        let content = MemberContent{
            draft_list: results?
        };
        Ok(content)
    }

  
pub fn member_flows(
    conn: &mut DbConn,
    host_id: i32,
    member_id: i32,
) -> Result<MemberFlows, AppError> {
    let member_uuid = LedgerService::get_user_entity_id(conn, host_id, member_id)?;
    let uuid = Uuid::parse_str(member_uuid.as_str())
    .map_err(|e| AppError::User(format!("Bad UUID: {}", e)))?;


let (inflows, inflow_entities, _) = LedgerService::get_flow_events(
    conn,
    FlowQuery::new(host_id).entity(uuid).to(),
)?;

let (outflows, outflow_entities, _) = LedgerService::get_flow_events(
    conn,
    FlowQuery::new(host_id).entity(uuid).from(),
)?;

// Merge entities, deduplicating by id
let mut entity_map: HashMap<String, EntityRef> = HashMap::new();

for e in inflow_entities.into_iter().chain(outflow_entities.into_iter()) {
    entity_map.entry(e.id.clone()).or_insert(e);
}

let entities: Vec<EntityRef> = entity_map.into_values().collect();



    Ok(MemberFlows {
        member_uuid,
        inflows,
        outflows,
        entities,
    })
}
}
