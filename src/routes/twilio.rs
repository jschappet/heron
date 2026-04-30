use actix_web::{HttpResponse, Responder, Scope, web};
use actix_web::web::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app_state::AppState;
use crate::routes::{register, RoutePath};
use crate::services::sms::{get_all_sms_replies, insert_sms_reply, send_sms};
use crate::types::method::Method;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsGateWebhook {
    pub device_id: String,
    pub event: String,
    pub id: String,
    pub payload: SmsGatePayload,
    pub webhook_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsGatePayload {
    pub message_id: String,
    pub body: String,
    pub sender: String,
    pub recipient: String,
    pub phone_number: String,
    pub sim_number: i32,
    pub received_at: String,
    pub subject: String,
    pub attachments: Vec<Value>,
}

#[derive(Deserialize, Serialize)]
struct SendSmsRequest {
    to: String,
    body: String,
}

async fn get_sms_replies(data: web::Data<AppState>) -> impl Responder {
    let mut conn = data.db_pool.get().expect("Database connection failed");
    match get_all_sms_replies(&mut conn) {
        Ok(replies) => HttpResponse::Ok().json(replies),
        Err(e) => {
            eprintln!("DB query error: {:?}", e);
            HttpResponse::InternalServerError().body("Error fetching SMS replies")
        }
    }
}

async fn receive_sms_reply(in_body: Bytes, data: web::Data<AppState>) -> impl Responder {
    let raw = String::from_utf8_lossy(&in_body).to_string();
    log::info!("SMSGate webhook received: {}", raw);

    let webhook: SmsGateWebhook = match serde_json::from_slice(&in_body) {
        Ok(w) => w,
        Err(e) => {
            log::warn!("Failed to parse SMSGate payload: {}", e);
            return HttpResponse::BadRequest().body("Invalid payload");
        }
    };

    let mut conn = data.db_pool.get().expect("Database connection failed");
    match insert_sms_reply(
        &mut conn,
        webhook.payload.sender,
        webhook.payload.recipient,
        webhook.payload.body,
        Some(raw),
    ) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("DB insert error: {:?}", e);
            HttpResponse::InternalServerError().body("Error storing message")
        }
    }
}

async fn send_sms_api(
    form_data: web::Json<SendSmsRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match send_sms(&data.settings.smsgate, &form_data.to, &form_data.body).await {
        Ok(_) => HttpResponse::Ok().body("SMS sent"),
        Err(e) => {
            eprintln!("Error sending SMS: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to send SMS")
        }
    }
}

pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register(
            "replies",
            Method::GET,
            path.as_str(),
            "",
            get_sms_replies,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "webhook",
            Method::POST,
            path.as_str(),
            "webhook",
            receive_sms_reply,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "send",
            Method::POST,
            path.as_str(),
            "send",
            send_sms_api,
            crate::types::MemberRole::Admin,
        ))
}
