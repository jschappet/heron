use diesel::prelude::*;
//use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::contribution_events;

//use crate::validator::AuthenticatedUser;
//use crate::schema::wants_to_contribute::dsl::wants_to_contribute;
// ---------- WantsToHelp ----------

#[derive(Debug, Queryable, Identifiable, Serialize)]
#[diesel(table_name = crate::schema::wants_to_contribute)]

pub struct WantsToContribute {
    pub id: i32,
    pub offer_id: i32,
    pub helper_user_id: i32,
    pub who: Option<String>,
    pub how_helping: Option<String>,
    pub availability_days: Option<String>,
    pub availability_times: Option<String>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::wants_to_contribute)]
pub struct WantsToContributeInput {
    pub offer_id: i32,
    pub helper_user_id: i32,
    pub who: Option<String>,
    pub how_helping: Option<String>,
    pub availability_days: Option<String>,
    pub availability_times: Option<String>,
    pub notes: Option<String>,
}

// ---------- HelpEvent ----------

// ---------- ContributeEvent ----------

#[derive(Debug, Queryable, Identifiable, Serialize, Selectable)]
#[diesel(table_name = crate::schema::contribution_events)]
pub struct ContributionEvent {
 
    pub id: i32,

    pub context_id: i32,
    pub contributor_id: i32,

    pub effort_date: Option<NaiveDateTime>,
    pub hours: Option<f32>,

    pub work_done: String,
    pub details: String,

    pub appreciation_message: String,
    pub public_flag: bool,

    pub created_at: NaiveDateTime,
}


#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::contribution_events)]
pub struct ContributionEventInput {
    pub context_id: i32,
    pub contributor_id: i32,

    pub effort_date: Option<NaiveDateTime>,
    pub hours: Option<f32>,

    pub work_done: String,
    pub details: Option<String>,

    pub appreciation_message: Option<String>,
    pub public_flag: Option<bool>,
}

