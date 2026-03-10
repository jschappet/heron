use crate::db::DbConn;
use crate::errors::app_error::AppError;
use crate::models::entities::{Entity, EntityUser, NewEntity, NewEntityUser};
use crate::models::flow_events::{FlowEvent, NewFlowEvent};
use crate::schema::flow_events::host_id;
use crate::schema::{entities, entity_users, flow_events};
use crate::types::JsonField;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{alias, prelude::*};
use serde::Serialize;
use uuid::Uuid;

/// Service layer for interacting with entities and flow events in the ledger.
pub struct LedgerService;

#[derive(Serialize, Clone)]
pub struct LedgerEventDto {
    pub id: String,
    pub timestamp: NaiveDateTime,
    pub resource_type: String,
    pub quantity_value: f32,
    pub quantity_unit: String,

    pub from: EntityRef,
    pub to: EntityRef,
}

#[derive(Serialize, Clone)]
pub struct EntityRef {
    pub id: String,
    pub name: String,
    pub entity_type: String,
}

#[derive(Queryable)]
pub struct LedgerEventRow {
    pub id: String,
    pub timestamp: NaiveDateTime,
    pub resource_type: String,
    pub quantity_value: f32,
    pub quantity_unit: String,

    pub from_id: String,
    pub from_name: String,
    pub from_type: String,

    pub to_id: String,
    pub to_name: String,
    pub to_type: String,
}

impl From<LedgerEventRow> for LedgerEventDto {
    fn from(row: LedgerEventRow) -> Self {
        LedgerEventDto {
            id: row.id,
            timestamp: row.timestamp,
            resource_type: row.resource_type,
            quantity_value: row.quantity_value,
            quantity_unit: row.quantity_unit,
            from: EntityRef {
                id: row.from_id,
                name: row.from_name,
                entity_type: row.from_type,
            },
            to: EntityRef {
                id: row.to_id,
                name: row.to_name,
                entity_type: row.to_type,
            },
        }
    }
}

impl LedgerService {
    // ----------------------------
    // ENTITY CRUD
    // ----------------------------
    pub fn create_entity(conn: &mut DbConn, new: NewEntity) -> Result<Entity, AppError> {
        diesel::insert_into(entities::table)
            .values(&new)
            .execute(conn)
            .map_err(|e| AppError::User(e.to_string()))?;

        entities::table
            .find(&new.id)
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn create_entity_user(conn: &mut DbConn, new: NewEntityUser) -> Result<EntityUser, AppError> {
        //use crate::schema::entity_users::dsl::*;

        diesel::insert_into(entity_users::table)
            .values(&new)
            .execute(conn)
            .map_err(|e| AppError::User(e.to_string()))?;

        
        entity_users::table
            .filter(entity_users::entity_id.eq(new.entity_id))
            .filter(entity_users::user_id.eq(new.user_id))
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn get_entity(conn: &mut DbConn, id: &str) -> Result<Entity, AppError> {
        entities::table.find(id).first(conn).map_err(|e| e.into())
    }


    pub fn save_all_entries(conn: &mut DbConn, payload: Vec<NewFlowEvent>) -> Result<String, AppError> {

        let r = conn.transaction(|conn| {
            diesel::insert_into(flow_events::table)
                .values(&payload)
                .execute(conn)
            });
        Ok("saved".to_string())
    }

    pub fn get_user_entity_id(conn: &mut DbConn, host: i32, user: i32) -> Result<String, AppError> {
        use crate::schema::{entities::dsl as e, entity_users::dsl as eu};

        let entity_id_result = eu::entity_users
            .inner_join(e::entities.on(e::id.eq(eu::entity_id)))
            .filter(eu::user_id.eq(user))
            .filter(e::host_id.eq(host))
            .select(eu::entity_id)
            .first::<String>(conn);

        let system_entity_id = format!("system_{}", host);

        match entity_id_result {
            Ok(id) => Ok(id),
            Err(diesel::result::Error::NotFound) => {
                // create entity
                let new_entity = NewEntity {
                    id: Uuid::new_v4().to_string(),
                    name: format!("User {}", user),
                    entity_type: "Person".to_string(),
                    host_id: host,
                    created_by: system_entity_id,
                    created_at: chrono::Utc::now().naive_utc(),
                    details: JsonField::default(),
                };
                let entity = Self::create_entity(conn, new_entity)?;

                // create entity_user
                let new_entity_user = NewEntityUser {
                    entity_id: &entity.id,
                    user_id: user,
                    role: "member",
                    status: "active",
                };
                Self::create_entity_user(conn, new_entity_user)?;

                Ok(entity.id)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn find_entity_by_name(
        conn: &mut DbConn,
        input_name: &str,
        host: i32,
    ) -> Result<Entity, AppError> {
        use crate::schema::entities::dsl::*;

        entities
            .filter(name.eq(input_name))
            .filter(host_id.eq(host))
            .first::<Entity>(conn)
            .map_err(|e| e.into())
    }



    pub fn get_entities(conn: &mut DbConn, host: i32) -> Result<Vec<Entity>, AppError> {
        entities::table
            .select(Entity::as_select())
            .filter(entities::host_id.eq(host))
            .load(conn)
            .map_err(|e| e.into())
    }

    pub fn update_entity(
        conn: &mut DbConn,
        id: &str,
        updated: NewEntity,
    ) -> Result<Entity, AppError> {
        diesel::update(entities::table.find(id))
            .set(&updated)
            .execute(conn)
            .map_err(|e| AppError::User(e.to_string()))?;

        entities::table.find(id).first(conn).map_err(|e| e.into())
    }

    // Optional: delete (or mark as retired)
    pub fn delete_entity(conn: &mut DbConn, id: &str) -> Result<usize, AppError> {
        diesel::delete(entities::table.find(id))
            .execute(conn)
            .map_err(|e| e.into())
    }

    // ----------------------------
    // FLOW EVENTS
    // ----------------------------
    pub fn create_flow_event(conn: &mut DbConn, new: NewFlowEvent) -> Result<FlowEvent, AppError> {
        diesel::insert_into(flow_events::table)
            .values(&new)
            .execute(conn)
            .map_err(|e| AppError::User(e.to_string()))?;

        flow_events::table
            .find(&new.id)
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn _get_flow_events(conn: &mut DbConn, host: i32) -> Result<Vec<FlowEvent>, AppError> {
        flow_events::table
            .order(flow_events::timestamp.asc())
            .select(FlowEvent::as_select())
            .filter(host_id.eq(host))
            .load(conn)
            .map_err(|e| e.into())
    }

    pub fn get_flow_events(
        conn: &mut DbConn,
        host: i32,
        entity_filter: Option<String>,
    ) -> Result<Vec<LedgerEventDto>, AppError> {
        let (from_entities, to_entities) =
            alias!(entities as from_entities, entities as to_entities);

        let mut query = flow_events::table
            .inner_join(
                from_entities.on(flow_events::from_entity.eq(from_entities.field(entities::id))),
            )
            .inner_join(to_entities.on(flow_events::to_entity.eq(to_entities.field(entities::id))))
            .filter(flow_events::host_id.eq(host))
            .into_boxed(); // ← allows conditional filters

        if let Some(entity_id) = entity_filter {
            query = query.filter(
                flow_events::from_entity
                    .eq(entity_id.clone())
                    .or(flow_events::to_entity.eq(entity_id.clone())),
            );
        }

        let rows: Vec<LedgerEventRow> = query
            .order(flow_events::timestamp.desc())
            .select((
                flow_events::id,
                flow_events::timestamp,
                flow_events::resource_type,
                flow_events::quantity_value,
                flow_events::quantity_unit,
                from_entities.field(entities::id),
                from_entities.field(entities::name),
                from_entities.field(entities::entity_type),
                to_entities.field(entities::id),
                to_entities.field(entities::name),
                to_entities.field(entities::entity_type),
            ))
            .load(conn)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub fn _old_get_flow_events(
        conn: &mut DbConn,
        host: i32,
    ) -> Result<Vec<LedgerEventDto>, AppError> {
        let (from_entities, to_entities) =
            alias!(entities as from_entities, entities as to_entities);
        let rows: Vec<LedgerEventRow> = flow_events::table
            .inner_join(
                from_entities.on(flow_events::from_entity.eq(from_entities.field(entities::id))),
            )
            .inner_join(to_entities.on(flow_events::to_entity.eq(to_entities.field(entities::id))))
            .filter(flow_events::host_id.eq(host))
            .order(flow_events::timestamp.desc())
            .select((
                flow_events::id,
                flow_events::timestamp,
                flow_events::resource_type,
                flow_events::quantity_value,
                flow_events::quantity_unit,
                from_entities.field(entities::id),
                from_entities.field(entities::name),
                from_entities.field(entities::entity_type),
                to_entities.field(entities::id),
                to_entities.field(entities::name),
                to_entities.field(entities::entity_type),
            ))
            .load(conn)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub fn get_inflows(conn: &mut DbConn, entity_id: &str) -> Result<Vec<FlowEvent>, AppError> {
        use crate::schema::flow_events::dsl::*;
        flow_events
            .filter(to_entity.eq(entity_id))
            .order(timestamp.asc())
            .load(conn)
            .map_err(|e| e.into())
    }

    pub fn get_outflows(conn: &mut DbConn, entity_id: &str) -> Result<Vec<FlowEvent>, AppError> {
        use crate::schema::flow_events::dsl::*;
        flow_events
            .filter(from_entity.eq(entity_id))
            .order(timestamp.asc())
            .load(conn)
            .map_err(|e| e.into())
    }

    pub fn get_flows_by_resource(
        conn: &mut DbConn,
        resource: &str,
    ) -> Result<Vec<FlowEvent>, AppError> {
        use crate::schema::flow_events::dsl::*;
        flow_events
            .filter(resource_type.eq(resource))
            .order(timestamp.asc())
            .load(conn)
            .map_err(|e| e.into())
    }
}
