use crate::{schema::{entities, entity_aliases, entity_users}, types::JsonField};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = entities)]
pub struct Entity {
    pub id: String,
    pub name: String,    
    pub entity_type: String,
    pub host_id: i32,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub details: JsonField,
    
}

#[derive(Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = entities)]
pub struct NewEntity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub host_id: i32,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub details: JsonField,
    
}

#[derive(Debug, Clone, Selectable, Queryable, Identifiable)]
#[diesel(table_name = entity_aliases)]
pub struct EntityAlias {
    pub id: String,
    pub entity_id: String,
    pub alias: String,
    pub created_by: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, AsChangeset )]
#[diesel(table_name = entity_aliases)]
pub struct NewEntityAlias<'a> {
    pub id: &'a str,
    pub entity_id: &'a str,
    pub alias: &'a str,
    pub created_by: &'a str,
}


#[derive(Debug, Clone, Selectable, Queryable, Identifiable)]
#[diesel(table_name = entity_users)]
pub struct EntityUser {
    pub id: i32,
    pub entity_id: String,
    pub user_id: i32,
    pub role: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = entity_users)]
pub struct NewEntityUser<'a> {
    pub entity_id: &'a str,
    pub user_id: i32,
    pub role: &'a str,
    pub status: &'a str,
}

pub fn create_entity(
    conn: &mut SqliteConnection,
    new: &NewEntity,
) -> QueryResult<Entity> {
    diesel::insert_into(entities::table)
        .values(new)
        .execute(conn)?;

    entities::table.find(&new.id).first(conn)
}

pub fn get_entities(conn: &mut SqliteConnection) -> QueryResult<Vec<Entity>> {
    entities::table.select(Entity::as_select()).load(conn)
}

pub fn get_entity(
    conn: &mut SqliteConnection,
    entity_id: &str,
) -> QueryResult<Entity> {
    entities::table.find(entity_id).first(conn)
}

pub fn update_entity(
    conn: &mut SqliteConnection,
    entity_id: &str,
    updated: &NewEntity,
) -> QueryResult<Entity> {
    diesel::update(entities::table.find(entity_id))
        .set(updated)
        .execute(conn)?;

    entities::table.find(entity_id).first(conn)
}

pub fn delete_entity(
    conn: &mut SqliteConnection,
    entity_id: &str,
) -> QueryResult<usize> {
    diesel::delete(entities::table.find(entity_id)).execute(conn)
}


pub fn create_entity_alias(
    conn: &mut SqliteConnection,
    new: &NewEntityAlias,
) -> QueryResult<EntityAlias> {
    diesel::insert_into(entity_aliases::table)
        .values(new)
        .execute(conn)?;

    entity_aliases::table.find(&new.id).first(conn)
}

pub fn get_entity_aliases(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<EntityAlias>> {
    entity_aliases::table
        .select(EntityAlias::as_select())
        .load(conn)
}

pub fn get_entity_alias(
    conn: &mut SqliteConnection,
    alias_id: &str,
) -> QueryResult<EntityAlias> {
    entity_aliases::table.find(alias_id).first(conn)
}

pub fn update_entity_alias(
    conn: &mut SqliteConnection,
    alias_id: &str,
    updated: &NewEntityAlias,
) -> QueryResult<EntityAlias> {
    diesel::update(entity_aliases::table.find(alias_id))
        .set(updated)
        .execute(conn)?;

    entity_aliases::table.find(alias_id).first(conn)
}

pub fn delete_entity_alias(
    conn: &mut SqliteConnection,
    alias_id: &str,
) -> QueryResult<usize> {
    diesel::delete(entity_aliases::table.find(alias_id)).execute(conn)
}



pub fn create_entity_user(
    conn: &mut SqliteConnection,
    new: &NewEntityUser,
) -> QueryResult<EntityUser> {
    diesel::insert_into(entity_users::table)
        .values(new)
        .execute(conn)?;

    entity_users::table
        .order(entity_users::id.desc())
        .first(conn)
}

pub fn get_entity_users(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<EntityUser>> {
    entity_users::table
        .select(EntityUser::as_select())
        .load(conn)
}

pub fn get_entity_user(
    conn: &mut SqliteConnection,
    entity_user_id: i32,
) -> QueryResult<EntityUser> {
    entity_users::table.find(entity_user_id).first(conn)
}

pub fn update_entity_user(
    conn: &mut SqliteConnection,
    entity_user_id: i32,
    updated: &NewEntityUser,
) -> QueryResult<EntityUser> {
    diesel::update(entity_users::table.find(entity_user_id))
        .set(updated)
        .execute(conn)?;

    entity_users::table.find(entity_user_id).first(conn)
}

pub fn delete_entity_user(
    conn: &mut SqliteConnection,
    entity_user_id: i32,
) -> QueryResult<usize> {
    diesel::delete(entity_users::table.find(entity_user_id)).execute(conn)
}

pub fn get_aliases_for_entity(
    conn: &mut SqliteConnection,
    entity_id: &str,
) -> QueryResult<Vec<EntityAlias>> {
    entity_aliases::table
        .filter(entity_aliases::entity_id.eq(entity_id))
        .select(EntityAlias::as_select())
        .load(conn)
}

pub fn get_users_for_entity(
    conn: &mut SqliteConnection,
    entity_id: &str,
) -> QueryResult<Vec<EntityUser>> {
    entity_users::table
        .filter(entity_users::entity_id.eq(entity_id))
        .select(EntityUser::as_select())
        .load(conn)
}