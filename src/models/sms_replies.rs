use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::sms_replies)]
pub struct SmsReply {
    pub id: i32,
    pub registration_id: Option<i32>,
    pub to_number: String,
    pub from_number: String,
    pub body: String,
    pub received_at: NaiveDateTime,
    pub parsed_response: Option<String>,
    pub raw_payload: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::sms_replies)]
pub struct NewSmsReply {
    pub registration_id: Option<i32>,
    pub from_number: String,
    pub to_number: String,
    pub body: String,
    pub received_at: NaiveDateTime,
    pub parsed_response: Option<String>,
    pub raw_payload: Option<String>,
}
