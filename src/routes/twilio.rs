use actix_web::{HttpResponse, Responder, Scope, get, post, web};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use crate::{app_state::AppState, routes::register, schema::sms_replies::dsl::*, types::method::Method};

use serde::{Deserialize, Serialize};
use reqwest::Client;

// Diesel model
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

// Struct for insert
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

// Deserialize incoming Twilio webhook
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct TwilioSmsPayload {
    pub From: String,
    pub To: String,
    pub Body: String,
}

// DB insert function
pub fn insert_sms_reply(
    conn: &mut SqliteConnection,
    from: String,
    to: String,
    other_body: String,
) -> QueryResult<SmsReply> {


    let new_reply = NewSmsReply {
        from_number: from,
        registration_id: None, // Set later if needed
        to_number: to,
        body: other_body,
        received_at: Utc::now().naive_utc(),
        parsed_response: None, // Optional, can be set later
        raw_payload: None, // Optional, can be set later
    };

    diesel::insert_into(sms_replies)
        .values(&new_reply)
        .execute(conn)?;

    sms_replies
        .order(id.desc())
        .first::<SmsReply>(conn)
}

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

// Actix handler
// #[post("/webhook")]
async fn receive_sms_reply(
    form: web::Form<TwilioSmsPayload>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut conn  = &mut data.db_pool.get()
        .expect("Database connection failed");
    match insert_sms_reply(
        &mut conn,
        form.From.clone(),
        form.To.clone(),
        form.Body.clone(),
    ) {
        Ok(_) => {
            // Create a response to Twilio
            let response = r###"<?xml version="1.0" encoding="UTF-8"?>
                <Response>
                    <Message>Thank you for your message! We will get back to you shortly.</Message>
                </Response>"###;
            HttpResponse::Ok()
                .content_type("application/xml")
                .body(response)

        },
        Err(e) => {
            eprintln!("DB insert error: {:?}", e);
            HttpResponse::InternalServerError().body("Error storing message")
        }
    }
}



pub async fn send_sms(
    data: web::Data<AppState>,
    to: &str,
    other_body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let account_sid = &data.settings.twilio.account_sid;
    let auth_token = &data.settings.twilio.auth_token;
    let from = &data.settings.twilio.phone_number;
    // Construct the Twilio API URL

    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let client = Client::new();

    let params = [
        ("To", to),
        ("From", from),
        ("Body", other_body),
    ];

    let res = client
        .post(&url)
        .basic_auth(account_sid, Some(auth_token))
        .form(&params)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        let text = res.text().await?;
        Err(format!("Twilio API error: {}", text).into())
    }
}


// #[post("/send_sms")]
async fn send_sms_api(
    form_data: web::Json<SendSmsRequest>,
    data: web::Data<AppState>,

) -> impl Responder {
    match send_sms(data, &form_data.to, &form_data.body).await {
        Ok(_) => HttpResponse::Ok().body("SMS sent"),
        Err(e) => {
            eprintln!("Error sending SMS: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to send SMS")
        }
    }
}

#[derive(Deserialize)]
struct SendSmsRequest {
    to: String,
    body: String,
}


pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        .service(register(
            "replies",
            Method::GET,
            &full_path,
            "",
            get_sms_replies,
            crate::types::MemberRole::Admin,
        ))

        // Twilio webhook (public — Twilio must reach it)
        .service(register(
            "webhook",
            Method::POST,
            &full_path,
            "",
            receive_sms_reply,
            crate::types::MemberRole::Public,
        ))

        // Admin: send outbound SMS
        .service(register(
            "send",
            Method::POST,
            &full_path,
            "",
            send_sms_api,
            crate::types::MemberRole::Admin,
        ))

// .service(receive_sms_reply)
//         .service(send_sms_api)
//         .service(get_sms_replies)
}
