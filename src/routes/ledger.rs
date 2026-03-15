use crate::domains::ledger_domain::{LedgerDomain};
use crate::middleware::host::{HostContext};


use crate::models::entities::NewEntity;
use crate::models::flow_events::NewFlowEvent;
use crate::routes::register;
use crate::types::JsonField;
//use crate::services::hosts::HostDomain;
use crate::types::method::Method;
use crate::validator::AuthContext;
use actix_web::{HttpResponse, Responder, Scope, web};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct NewFlowPayload {
    pub from_entity: String,
    pub to_entity: String,
    pub resource_type: String,
    pub quantity_value: f32,
    pub quantity_unit: String,
    pub notes: Option<String>,
    pub timestamp: Option<NaiveDateTime>,
    pub details: JsonField,
}



#[derive(Debug, Deserialize)]
pub struct NewEntityPayload {
    pub name: String,
    pub entity_type: String,
    pub details: JsonField,
}

// -----------------------------
// ENTITY ROUTES
// -----------------------------
async fn create_entity(
    domain: web::Data<LedgerDomain>,
    payload: web::Json<NewEntityPayload>,
    host: HostContext, 
    auth: AuthContext,
) -> impl Responder {

    let created_by = domain.get_user_entity_id(host.0.id, auth.user_id).unwrap();

    let new_entity = crate::models::entities::NewEntity {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name.clone(),
        host_id: host.0.id,
        entity_type: payload.entity_type.clone(),
        created_at: chrono::Utc::now().naive_utc(),
        created_by: created_by,
        details: payload.details.clone(),
    };

    match domain.create_entity(new_entity) {
        Ok(entity) => HttpResponse::Ok().json(entity),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_entities(
    domain: web::Data<LedgerDomain>,
    //host: web::Data<crate::middleware::host::HostContext>,
    host: HostContext
) -> impl Responder {
    match domain.get_all_entities(host.0.id) {
        Ok(entities) => HttpResponse::Ok().json(entities),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_entity(
    domain: web::Data<LedgerDomain>,
    path: web::Path<String>,
) -> impl Responder {
    let entity_id = path.into_inner();
    match domain.get_entity(&entity_id) {
        Ok(entity) => HttpResponse::Ok().json(entity),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


#[derive(Debug, Deserialize)]
struct NewBulkEntityPayload {
    rows: Vec<NewEntityPayload> 
}

async fn submit_bulk_entities(
    domain: web::Data<LedgerDomain>,
    payload: web::Json<NewBulkEntityPayload>,
    auth: AuthContext,
    host: HostContext
)-> impl Responder {

   
    let created_by = domain.get_user_entity_id(host.0.id, auth.user_id).unwrap();
    
    let events: Vec<NewEntity> = payload.rows.iter().map(|row| {
        NewEntity {
            id: uuid::Uuid::new_v4().to_string(),
            name: row.name.clone(),
            host_id: host.0.id,
            entity_type: row.entity_type.clone(),
            created_at: chrono::Utc::now().naive_utc(),
            created_by: created_by.clone(),
            details: row.details.clone(),
        }
    }).collect();

   let results = domain.save_all_entities(events);
   format!("{}",json!({"results": results.unwrap_or_default() }))
}

#[derive(Debug, Deserialize)]
struct NewBulkFlowPayload {
    rows: Vec<NewFlowPayload> 
}

async fn submit_bulk_flows(
    domain: web::Data<LedgerDomain>,
    payload: web::Json<NewBulkFlowPayload>,
    auth: AuthContext,
    host: HostContext
)-> impl Responder {

   
    let entity_user_id = domain.resolve_or_create_entity(auth.user_id, host.0.id).unwrap();


    let events: Vec<NewFlowEvent> = payload.rows.iter().map(|row| {
        NewFlowEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: row
                .timestamp
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            from_entity: row.from_entity.clone(),
            to_entity: row.to_entity.clone(),
            host_id: host.0.id.clone(),

            resource_type: row.resource_type.clone(),
            quantity_value: row.quantity_value,
            quantity_unit: row.quantity_unit.clone(),

            notes: row.notes.clone(),
            recorded_at: chrono::Utc::now().naive_utc(),
            details: row.details.clone(),

            created_by: entity_user_id.clone(),
        }
    }).collect();

   let results = domain.save_all_flow_events(events);
   format!("{}",json!({"results": results.unwrap_or_default() }))
}

// -----------------------------
// FLOW EVENT ROUTES
// -----------------------------
async fn submit_flow(
    domain: web::Data<LedgerDomain>,
    payload: web::Json<NewFlowPayload>,
    auth: AuthContext,
    //host: web::Data<crate::middleware::host::HostContext>,
    host: HostContext
) -> impl Responder {
    let current_time = chrono::Utc::now().naive_utc();
    let timestamp = payload
        .timestamp
        .unwrap_or_else(|| current_time);

    let entity_user_id = domain.resolve_or_create_entity(auth.user_id, host.0.id).unwrap();

    let from_id = domain.resolve_entity(&payload.from_entity, host.0.id).unwrap();
    let to_id   = domain.resolve_entity(&payload.to_entity, host.0.id).unwrap();
    log::debug!("From: {:?}", from_id);
    log::debug!("To: {:?}", to_id);

    let new_flow = crate::models::flow_events::NewFlowEvent {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp,
        from_entity: from_id,
        to_entity: to_id,
        host_id: host.0.id,
        resource_type: payload.resource_type.clone(),
        quantity_value: payload.quantity_value,
        quantity_unit: payload.quantity_unit.clone(),
        notes: payload.notes.clone(),
        recorded_at: chrono::Utc::now().naive_utc(),
        details: payload.details.clone(),
        created_by: entity_user_id,
    };

    match domain.record_flow(new_flow) {
        Ok(flow) => HttpResponse::Ok().json(flow),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_entity_flows(
    domain: web::Data<LedgerDomain>,
    path: web::Path<String>,
    host: HostContext,

) -> impl Responder {
    let entity_id = path.into_inner();
    match domain.get_flows_for_entity(host.0.id, &entity_id) {
        Ok(flows) => HttpResponse::Ok().json(flows),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_ledger(
    domain: web::Data<LedgerDomain>,
    host: HostContext
) -> impl Responder {
    match domain.get_ledger_summary(host.0.id) {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// -----------------------------
// SCOPE REGISTRATION
// -----------------------------
pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");

    web::scope("")
        // Entities
        .service(register(
            "create_entity",
            Method::POST,
            &full_path,
            "entity",
            create_entity,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "get_entities",
            Method::GET,
            &full_path,
            "entities",
            get_entities,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "get_entity",
            Method::GET,
            &full_path,
            "entity/{id}",
            get_entity,
            crate::types::MemberRole::Public,
        ))
        // Flows
        .service(register(
            "submit_flow",
            Method::POST,
            &full_path,
            "flow",
            submit_flow,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "get_entity_flows",
            Method::GET,
            &full_path,
            "entity/{id}/flows",
            get_entity_flows,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "get_ledger",
            Method::GET,
            &full_path,
            "ledger.json",
            get_ledger,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "ledger_submit_bulk_flow",
            Method::POST,
            &full_path,
            "submit/bulk",
            submit_bulk_flows,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "submit_bulk_entities",
            Method::POST,
            &full_path,
            "submit/entities/bulk",
            submit_bulk_entities,
            crate::types::MemberRole::Admin,
        ))
        //
        //submit_bulk_flows
}