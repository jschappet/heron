use diesel::prelude::*;
//use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

//use crate::schema::effort_contexts;
// ---------- EffortContext ----------

#[derive(Debug, Queryable, Identifiable, Serialize)]
#[diesel(table_name = crate::schema::effort_contexts)]
pub struct EffortContext {
    pub id: i32,

    pub context_type: String,
    pub short_code: String,

    pub name: String,
    pub description: String,

    pub created_at: NaiveDateTime,
    pub active_flag: bool,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::effort_contexts)]
pub struct EffortContextInput {
    pub context_type: String,
    pub short_code: String,

    pub name: String,
    pub description: String,
    pub active_flag: bool,
}

