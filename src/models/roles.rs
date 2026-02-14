use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

use crate::schema::roles;

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub show_in_directory: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(AsChangeset, Insertable, Deserialize)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
    pub show_in_directory: bool,
}

pub fn create_role(conn: &mut SqliteConnection, new: &NewRole) -> QueryResult<Role> {
    diesel::insert_into(roles::table)
        .values(new)
        .execute(conn)?;

    roles::table.order(roles::id.desc()).first(conn)
}

pub fn load_roles(
    conn: &mut SqliteConnection,
    user_id: i32,
) -> QueryResult<Vec<String>> {
    use crate::schema::{memberships, roles};

    roles::table
        .inner_join(memberships::table.on(memberships::role_id.eq(roles::id)))
        .filter(memberships::user_id.eq(user_id))
        .filter(memberships::active.eq(true))
        .select(roles::name)
        .load::<String>(conn)
}


pub fn get_roles(conn: &mut SqliteConnection) -> QueryResult<Vec<Role>> {
    roles::table.load(conn)
}

pub fn get_role(conn: &mut SqliteConnection, role_id: i32) -> QueryResult<Role> {
    roles::table.find(role_id).first(conn)
}

pub fn update_role(
    conn: &mut SqliteConnection,
    role_id: i32,
    in_updated: &NewRole,
) -> QueryResult<Role> {
    diesel::update(roles::table.find(role_id))
        .set(in_updated)
        .execute(conn)?;

    roles::table.find(role_id).first(conn)
}

pub fn delete_role(conn: &mut SqliteConnection, role_id: i32) -> QueryResult<usize> {
    diesel::delete(roles::table.find(role_id)).execute(conn)
}
