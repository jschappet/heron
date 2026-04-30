use chrono::Utc;
use diesel::prelude::*;
use reqwest::Client;
use serde_json::json;

use crate::models::sms_replies::{NewSmsReply, SmsReply};
use crate::schema::sms_replies::dsl::*;
use crate::settings::SmsGate;

pub fn insert_sms_reply(
    conn: &mut SqliteConnection,
    from: String,
    to: String,
    message_body: String,
    raw: Option<String>,
) -> QueryResult<SmsReply> {
    let new_reply = NewSmsReply {
        from_number: from,
        registration_id: None,
        to_number: to,
        body: message_body,
        received_at: Utc::now().naive_utc(),
        parsed_response: None,
        raw_payload: raw,
    };

    diesel::insert_into(sms_replies)
        .values(&new_reply)
        .execute(conn)?;

    sms_replies.order(id.desc()).first::<SmsReply>(conn)
}

pub fn get_all_sms_replies(conn: &mut SqliteConnection) -> QueryResult<Vec<SmsReply>> {
    sms_replies
        .order(received_at.desc())
        .load::<SmsReply>(conn)
}

pub async fn send_sms(
    config: &SmsGate,
    to: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/message", config.host);

    let payload = json!({
        "textMessage": { "text": message },
        "phoneNumbers": [to]
    });

    let client = Client::new();
    log::info!("Sending SMS to {} via SmsGate at {}", to, url);
    log::info!("Sending SMS to user:{} password:{}", config.username, config.password);
    let res = client
        .post(&url)
        .basic_auth(&config.username, Some(&config.password))
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        log::info!("SMS sent successfully to {}", res.text().await?);
        Ok(())
    } else {
        let text = res.text().await?;
        Err(format!("SmsGate error: {}", text).into())
    }
}
