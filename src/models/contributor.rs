use diesel::prelude::*;
//use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

use crate::schema::contributors;
// ---------- Contributor ----------

#[derive(Debug, Queryable, Identifiable, Serialize)]
#[diesel(table_name = crate::schema::contributors)]
pub struct Contributor {
    pub id: i32,

    pub name: Option<String>,
    pub email: Option<String>,

    pub user_id: Option<i32>,

    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::contributors)]
pub struct ContributorInput {
    pub name: Option<String>,
    pub email: Option<String>,
    pub user_id: Option<i32>,
}
