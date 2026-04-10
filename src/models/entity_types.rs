use crate::schema::entity_types;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = entity_types)]
pub struct EntityType {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host_id: i32,
    pub can_hold_resources: bool,
    pub can_initiate_flows: bool,
    pub can_receive_flows: bool,
}

#[derive(Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = entity_types)]
pub struct NewEntityType {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host_id: i32,
    pub can_hold_resources: bool,
    pub can_initiate_flows: bool,
    pub can_receive_flows: bool,
}

pub fn create_entity_type(
    conn: &mut SqliteConnection,
    new: &NewEntityType,
) -> QueryResult<EntityType> {
    diesel::insert_into(entity_types::table)
        .values(new)
        .execute(conn)?;

    entity_types::table.find(&new.id).first(conn)
}

pub fn get_entity_types(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<EntityType>> {
    entity_types::table
        .select(EntityType::as_select())
        .load(conn)
}

pub fn get_entity_type(
    conn: &mut SqliteConnection,
    type_id: &str,
) -> QueryResult<EntityType> {
    entity_types::table.find(type_id).first(conn)
}