use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

use crate::schema::memberships;

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct Membership {
    pub id: i32,
    pub user_id: i32,
    pub role_id: i32,
    pub host_id: i32,

    pub active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub ended_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = memberships)]
pub struct NewMembership {
    pub user_id: i32,
    pub role_id: i32,
    pub host_id: i32,
    pub active: bool,
    
}

pub fn create_membership(
    conn: &mut SqliteConnection,
    new: &NewMembership,
) -> QueryResult<Membership> {
    diesel::insert_into(memberships::table)
        .values(new)
        .execute(conn)?;

    memberships::table.order(memberships::id.desc()).first(conn)
}

pub fn get_memberships(conn: &mut SqliteConnection) -> QueryResult<Vec<Membership>> {
    memberships::table.load(conn)
}

pub fn get_memberships_for_user(
    conn: &mut SqliteConnection,
    user_id: i32,
) -> QueryResult<Vec<Membership>> {
    memberships::table
        .filter(memberships::user_id.eq(user_id))
        .load(conn)
}

pub fn _deactivate_membership(conn: &mut SqliteConnection, id: i32) -> QueryResult<Membership> {
    diesel::update(memberships::table.find(id))
        .set(memberships::active.eq(false))
        .execute(conn)?;

    memberships::table.find(id).first(conn)
}

pub fn delete_membership(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(memberships::table.find(id)).execute(conn)
}
