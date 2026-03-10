use crate::{schema::flow_actions, schema::flow_events, types::JsonField};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = flow_events)]
pub struct FlowEvent {
    pub id: String,
    pub timestamp: NaiveDateTime,
    pub recorded_at: NaiveDateTime,
    pub from_entity: String,
    pub to_entity: String,
    pub host_id: i32,
    pub resource_type: String,
    pub quantity_value: f32,
    pub quantity_unit: String,
    pub notes: Option<String>,
    pub details: JsonField,
    pub created_by: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = flow_events)]
pub struct NewFlowEvent {
    pub id: String,
    pub timestamp: NaiveDateTime,
    pub recorded_at: NaiveDateTime,
    pub from_entity: String,
    pub to_entity: String,
    pub host_id: i32,
    pub resource_type: String,
    pub quantity_value: f32,
    pub quantity_unit: String,
    pub notes: Option<String>,
    pub details: JsonField,
    pub created_by: String,
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable)]
#[diesel(table_name = flow_actions)]
pub struct FlowAction {
    pub id: String,
    pub flow_id: String,
    pub action_type: String,
    pub actor_entity: String,
    pub timestamp: NaiveDateTime,
    pub details: String,
}

#[derive(Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = flow_actions)]
pub struct NewFlowAction<'a> {
    pub id: &'a str,
    pub flow_id: &'a str,
    pub action_type: &'a str,
    pub actor_entity: &'a str,
    pub details: &'a str,
}

pub fn create_flow_event(
    conn: &mut SqliteConnection,
    new: &NewFlowEvent,
) -> QueryResult<FlowEvent> {
    diesel::insert_into(flow_events::table)
        .values(new)
        .execute(conn)?;

    flow_events::table.find(&new.id).first(conn)
}

pub fn get_flow_events(conn: &mut SqliteConnection) -> QueryResult<Vec<FlowEvent>> {
    flow_events::table
        .order(flow_events::timestamp.asc())
        .select(FlowEvent::as_select())
        .load(conn)
}

pub fn get_flows_for_entity(
    conn: &mut SqliteConnection,
    entity_id: &str,
) -> QueryResult<Vec<FlowEvent>> {
    use crate::schema::flow_events::dsl::*;

    flow_events
        .filter(from_entity.eq(entity_id).or(to_entity.eq(entity_id)))
        .order(timestamp.asc())
        .load(conn)
}

pub fn get_flows_by_resource(
    conn: &mut SqliteConnection,
    resource: &str,
) -> QueryResult<Vec<FlowEvent>> {
    use crate::schema::flow_events::dsl::*;

    flow_events
        .filter(resource_type.eq(resource))
        .order(timestamp.asc())
        .load(conn)
}

pub fn create_flow_action(
    conn: &mut SqliteConnection,
    new: &NewFlowAction,
    action_id: &str, // pass UUID here
) -> QueryResult<FlowAction> {
    use crate::schema::flow_actions::dsl::*;

    diesel::insert_into(flow_actions)
        .values((
            id.eq(action_id),
            flow_id.eq(new.flow_id),
            action_type.eq(new.action_type),
            actor_entity.eq(new.actor_entity),
            details.eq(new.details),
        ))
        .execute(conn)?;

    flow_actions
        .filter(id.eq(action_id))
        .select(FlowAction::as_select())
        .first(conn)
}

pub fn get_flow_actions(conn: &mut SqliteConnection) -> QueryResult<Vec<FlowAction>> {
    use crate::schema::flow_actions::dsl::*;
    flow_actions.select(FlowAction::as_select()).load(conn)
}

pub fn get_flow_action(
    conn: &mut SqliteConnection,
    action_id_str: &str,
) -> QueryResult<FlowAction> {
    use crate::schema::flow_actions::dsl::*;
    flow_actions
        .filter(id.eq(action_id_str))
        .select(FlowAction::as_select())
        .first(conn)
}

pub fn update_flow_action(
    conn: &mut SqliteConnection,
    action_id_str: &str,
    updated: &NewFlowAction,
) -> QueryResult<FlowAction> {
    use crate::schema::flow_actions::dsl::*;

    diesel::update(flow_actions.filter(id.eq(action_id_str)))
        .set(updated)
        .execute(conn)?;

    flow_actions
        .filter(id.eq(action_id_str))
        .select(FlowAction::as_select())
        .first(conn)
}

pub fn delete_flow_action(conn: &mut SqliteConnection, action_id_str: &str) -> QueryResult<usize> {
    use crate::schema::flow_actions::dsl::*;
    diesel::delete(flow_actions.filter(id.eq(action_id_str))).execute(conn)
}
