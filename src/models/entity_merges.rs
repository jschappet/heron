use crate::schema::entity_merges;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = entity_merges)]
pub struct EntityMerge {
    pub id: String,
    pub from_entity: String,
    pub to_entity: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = entity_merges)]
pub struct NewEntityMerge {
    pub id: String,
    pub from_entity: String,
    pub to_entity: String,
}

pub fn create_entity_merge(
    conn: &mut SqliteConnection,
    new: &NewEntityMerge,
) -> QueryResult<EntityMerge> {
    diesel::insert_into(entity_merges::table)
        .values(new)
        .execute(conn)?;

    entity_merges::table.find(&new.id).first(conn)
}

pub fn get_entity_merges(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<EntityMerge>> {
    entity_merges::table
        .select(EntityMerge::as_select())
        .load(conn)
}