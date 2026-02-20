use actix_web::{HttpResponse, Responder, Scope, get, web};
//use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use crate::{app_state::AppState, schema::sms_replies::dsl::*, routes::twilio::SmsReply};

use crate::routes::register;
use crate::types::method::Method;

//use serde::{Deserialize, Serialize};
//use reqwest::Client;
// TODO Create a sms_replies view to return all replies
// this should display all of the replies and include options to send a reply back to the user
// 
fn get_all_sms_replies(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<SmsReply>> {
    sms_replies
        .order(received_at.desc())
        .load::<SmsReply>(conn)
}

// TODO create api endpoint to get all sms replies
// This will be used to display all replies in the admin panel
// #[get("/sms_replies")]
async fn get_sms_replies(
    data: web::Data<AppState>,
) -> impl Responder {
    let mut conn = data.db_pool.get().expect("Database connection failed");
    match get_all_sms_replies(&mut conn) {
        Ok(replies) => {
            HttpResponse::Ok().json(replies)
        },
        Err(e) => {
            eprintln!("DB query error: {:?}", e);
            HttpResponse::InternalServerError().body("Error fetching SMS replies")
        }
    }
}



pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
    .service(register(
    "get_sms_replies",
    Method::GET,
    &full_path,
    "",
    get_sms_replies,
    crate::types::MemberRole::Public,
))
    
}


