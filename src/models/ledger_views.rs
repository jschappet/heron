use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
pub struct VitalSigns {
    pub dollars_available: f64,
    pub active_contributors_30d: i32,
    pub total_hours_30d: f64,
    pub active_projects_30d: i32,
}

// API function to get the current vital signs
pub fn get_vital_signs(conn: &mut SqliteConnection) -> QueryResult<VitalSigns> {
    diesel::sql_query("SELECT * FROM v_regen_vital_signs")
        .get_result::<VitalSigns>(conn)
}

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct EntityBalance {
    pub entity_id: String,
    pub resource_type: String,
    pub balance: f64,
}

pub fn get_entity_balances(conn: &mut SqliteConnection) -> QueryResult<Vec<EntityBalance>> {
    diesel::sql_query("SELECT * FROM v_entity_balances")
        .load::<EntityBalance>(conn)
}


#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct DollarsAvailable {
    pub dollars_available: f64,
}

pub fn get_dollars_available(conn: &mut SqliteConnection) -> QueryResult<DollarsAvailable> {
    diesel::sql_query("SELECT * FROM v_dollars_available")
        .get_result::<DollarsAvailable>(conn)
}


#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct ActiveContributors {
    pub active_contributors: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct TotalHours {
    pub total_hours: f64,
}

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct ActiveProjects {
    pub active_projects: i32,
}


pub fn get_active_contributors(conn: &mut SqliteConnection) -> QueryResult<ActiveContributors> {
    diesel::sql_query("SELECT * FROM v_active_contributors_30d")
        .get_result::<ActiveContributors>(conn)
}

pub fn get_total_hours(conn: &mut SqliteConnection) -> QueryResult<TotalHours> {
    diesel::sql_query("SELECT * FROM v_total_hours_30d")
        .get_result::<TotalHours>(conn)
}

pub fn get_active_projects(conn: &mut SqliteConnection) -> QueryResult<ActiveProjects> {
    diesel::sql_query("SELECT * FROM v_active_projects_30d")
        .get_result::<ActiveProjects>(conn)
}