use crate::schema::entity_identities;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = entity_identities)]
pub struct EntityIdentity {
    pub id: String,
    pub entity_id: String,
    pub host_id: i32,
    pub identity_type: String,
    pub identity_value: String,
    pub verified: bool,
    pub primary_flag: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = entity_identities)]
pub struct NewEntityIdentity {
    pub id: String,
    pub entity_id: String,
    pub host_id: i32,
    pub identity_type: String,
    pub identity_value: String,
    pub verified: bool,
    pub primary_flag: bool,
}

pub fn create_entity_identity(
    conn: &mut SqliteConnection,
    new: &NewEntityIdentity,
) -> QueryResult<EntityIdentity> {
    diesel::insert_into(entity_identities::table)
        .values(new)
        .execute(conn)?;

    entity_identities::table.find(&new.id).first(conn)
}

pub fn get_entity_identities(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<EntityIdentity>> {
    entity_identities::table
        .select(EntityIdentity::as_select())
        .load(conn)
}

pub fn get_entity_identity(
    conn: &mut SqliteConnection,
    identity_id: &str,
) -> QueryResult<EntityIdentity> {
    entity_identities::table.find(identity_id).first(conn)
}

pub fn get_identities_for_entity(
    conn: &mut SqliteConnection,
    entity_id: &str,
) -> QueryResult<Vec<EntityIdentity>> {
    entity_identities::table
        .filter(entity_identities::entity_id.eq(entity_id))
        .select(EntityIdentity::as_select())
        .load(conn)
}

pub fn find_identity(
    conn: &mut SqliteConnection,
    identity_type: &str,
    identity_value: &str,
) -> QueryResult<EntityIdentity> {
    entity_identities::table
        .filter(entity_identities::identity_type.eq(identity_type))
        .filter(entity_identities::identity_value.eq(identity_value))
        .first(conn)
}