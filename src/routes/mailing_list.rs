use actix_web::{HttpRequest, HttpResponse, Responder, Scope, get, post, web};
//use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use chrono::{NaiveDateTime, Utc, Duration};
use diesel::prelude::*;
//use diesel::sqlite::SqliteConnection;
//use image::{ImageFormat, Luma};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
//use uuid::Uuid;

use crate::app_state::AppState;
use crate::middleware::host_utils::require_host_id;
// use crate::registration::Registration;
//use crate::schema::mailing_list_subscribers;
use crate::schema::mailing_list_subscribers::dsl::*;
use crate::settings::Settings;
use crate::validator::AuthContext;
//use crate::{generate_ticket_ids, registration, users};
use hmac::{Hmac, Mac};
use base64::{engine::general_purpose, Engine as _};

use lettre::{Message, SmtpTransport, Transport};
use lettre::message::Mailbox;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::mailing_list_subscribers)]
pub struct Subscriber {
    pub id: i32,
    pub host_id: i32,
    pub name: String,
    pub email: String,
    pub confirmed: bool,
    pub confirmation_token: Option<String>,
    pub unsubscribed: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mailing_list_subscribers)]
pub struct NewSubscriber<'a> {
    pub host_id: i32,
    pub name: &'a str,
    pub email: &'a str,
    pub confirmation_token: Option<&'a str>,
}

#[derive(Deserialize)]
pub struct SubscribeForm {
    pub name: String,
    pub email: String,
    pub nickname: Option<String>, // honeypot
}


#[derive(Serialize)]
pub struct PasswordResetContext<'a> {
    pub user_name: &'a str,
    pub reset_link: &'a str,
    pub site_name: &'a str,
}

// Helper: Generate a secure token with expiry
fn generate_token(other_email: &str, secret: &str, expiry_minutes: i64) -> String {
    let expiry = Utc::now() + Duration::minutes(expiry_minutes);
    let payload = format!("{}:{}", other_email, expiry.timestamp());
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    let sig = general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{}.{}.{}", other_email, expiry.timestamp(), sig)
}

// Helper: Validate token and check expiry
fn validate_token(token: &str, secret: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 { return None; }
    let other_email = parts[0];
    let expiry: i64 = parts[1].parse().ok()?;
    let sig = parts[2];
    if Utc::now().timestamp() > expiry { return None; }
    let payload = format!("{}:{}", other_email, expiry);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    let expected_sig = general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    if sig == expected_sig { Some(other_email.to_string()) } else { None }
}

// POST /subscribe
#[post("/subscribe")]
async fn subscribe(
    req: HttpRequest,
    data: web::Data<AppState>,
    form: web::Json <SubscribeForm>,
) -> impl Responder {

    // Get host ID — always returns a valid ID now
    let incoming_host_id = require_host_id(&req).await.unwrap(); // safe because fallback exists

    // Honeypot check
    if let Some(nick) = &form.nickname {
        if !nick.is_empty() {
            return HttpResponse::Ok().body("Bot detected.");
        }
    }
    // Basic validation
    if form.name.trim().is_empty() || form.email.trim().is_empty() || !form.email.contains('@') {
        return HttpResponse::BadRequest().body("Invalid input.");
    }
    let secret = std::env::var("MAILING_LIST_SECRET").unwrap_or_else(|_| "changeme".to_string());
    let token = generate_token(&form.email, &secret, 60 * 24); // 24h expiry
    

    let mut conn = data.db_pool.get().expect("DB connection failed");
    let existing = mailing_list_subscribers
        .filter(email.eq(&form.email))
        .filter(host_id.eq(incoming_host_id))
        .select(Subscriber::as_select())
        .first::<Subscriber>(&mut conn)
        .optional()
        .expect("DB error");
    if let Some( sub) = existing {
        log::info!("Updating existing subscriber: {}", sub.email);
        diesel::update(mailing_list_subscribers.filter(email.eq(&form.email)))
            .set((
                name.eq(&form.name),
                confirmation_token.eq(Some(token.clone())),
                unsubscribed.eq(false),
                confirmed.eq(false),
                host_id.eq(incoming_host_id),
            ))
            .execute(&mut conn)
            .expect("DB update error");
    } else {
        let new_sub = NewSubscriber {
            name: &form.name,
            email: &form.email,
            confirmation_token: Some(&token),
            host_id: incoming_host_id,
        };
        diesel::insert_into(mailing_list_subscribers)
            .values(&new_sub)
            .execute(&mut conn)
            .expect("DB insert error");
    }
    // TODO: Send confirmation email with link: /confirm/<token>
    // send_confirmation_email(&form.email, &token);

    HttpResponse::Ok().body("Check your email for a confirmation link.")
}

// GET /confirm/<token>
#[get("/confirm/{token}")]
async fn confirm(
    data: web::Data<AppState>,
    token: web::Path<String>,
) -> impl Responder {
    let secret = std::env::var("MAILING_LIST_SECRET").unwrap_or_else(|_| "changeme".to_string());
    if let Some(email_val) = validate_token(&token, &secret) {
        let mut conn = data.db_pool.get().expect("DB connection failed");
        let updated = diesel::update(mailing_list_subscribers.filter(email.eq(&email_val)))
            .set((confirmed.eq(true), confirmation_token.eq::<Option<String>>(None)))
            .execute(&mut conn)
            .expect("DB update error");
        if updated > 0 {
            return HttpResponse::Ok().body("Subscription confirmed! Thank you.");
        }
    }
    HttpResponse::BadRequest().body("Invalid or expired confirmation link.")
}

// GET /unsubscribe/<token>
#[get("/unsubscribe/{token}")]
async fn unsubscribe(
    data: web::Data<AppState>,
    token: web::Path<String>,
) -> impl Responder {
    let secret = std::env::var("MAILING_LIST_SECRET").unwrap_or_else(|_| "changeme".to_string());
    if let Some(email_val) = validate_token(&token, &secret) {
        let mut conn = data.db_pool.get().expect("DB connection failed");
        let updated = diesel::update(mailing_list_subscribers.filter(email.eq(&email_val)))
            .set(unsubscribed.eq(true))
            .execute(&mut conn)
            .expect("DB update error");
        if updated > 0 {
            return HttpResponse::Ok().body("You have been unsubscribed. Goodbye!");
        }
    }
    HttpResponse::BadRequest().body("Invalid or expired unsubscribe link.")
}

#[get("/mailing_list")]
async fn list_subscribers(
    req: HttpRequest,
    _auth: AuthContext,
    data: web::Data<AppState>
    ) -> impl Responder {
        // Get host ID — always returns a valid ID now
    let incoming_host_id = require_host_id(&req).await.unwrap(); // safe because fallback exists

    let mut conn = data.db_pool.get().expect("DB connection failed");
    let subs = mailing_list_subscribers
        .order(created_at.desc())
        .filter(host_id.eq(incoming_host_id))
        .load::<Subscriber>(&mut conn)
        .expect("DB query error");
    HttpResponse::Ok().json(subs)
}

pub fn send_templated_email<T: serde::Serialize>(
    to_email: &str,
    user_name: &str,
    subject: &str,
    html_template: &str,
    text_template: &str,
    context: &T,
    settings: &Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();

    handlebars.register_template_file("html", html_template)?;
    handlebars.register_template_file("text", text_template)?;

    let html_body = handlebars.render("html", context)?;
    let text_body = handlebars.render("text", context)?;

    let message = Message::builder()
        .from(settings.email.smtp_from_email.parse()?)
        .to(Mailbox::new(Some(user_name.to_string()), to_email.parse()?))
        .subject(subject)
        .multipart(
            lettre::message::MultiPart::alternative_plain_html(
                text_body,
                html_body,
            ),
        )?;

    let mailer = SmtpTransport::relay(settings.smtp.server.as_str())?
        .credentials((
            settings.smtp.username.as_str(),
            settings.smtp.password.as_str(),
        ).into())
        .build();

    mailer.send(&message)?;
    Ok(())
}


/// Sends a user account confirmation email with a verification link.
///
/// # Arguments
///
/// * `to_email` - Recipient's email address.
/// * `user_name` - Recipient's display name (for personalization).
/// * `token` - Verification token to include in the confirmation link.
///
/// # Errors
///
/// Returns any error that occurs during email construction or sending.
pub fn send_account_confirmation_email(
    to_email: &str,
    user_name: &str,
    token: &str,
    host_base_url: &str,
    host_display_name: &str,
    settings: &Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    let verification_link = format!(
        "{}/api/auth/token/{}",
        //settings.email.base_url,
        host_base_url,
        token
    );

    let context = AccountVerificationContext {
        user_name,
        verification_link: &verification_link,
        site_name: host_display_name
    };

    send_templated_email(
        to_email,
        user_name,
        format!("Verify Your {} Account", host_display_name).as_str(),
        "templates/email/verify_token.hbs",
        "templates/email/verify_token_text.hbs",
        &context,
        settings,
    )
}

/// Sends a password reset email with a reset link.
/// # Arguments
/// * `to_email` - Recipient's email address.
/// * `user_name` - Recipient's display
/// * `token` - Password reset token to include in the reset link.
/// # Errors
/// Returns any error that occurs during email construction or sending. 
///    
pub fn send_password_reset_email(
    to_email: &str,
    user_name: &str,
    token: &str,
    host_base_url: &str,
    host_display_name: &str,
    settings: &Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    let reset_link = format!(
        "{}/api/users/reset-password/{}",
        host_base_url,
        token
    );

    let context = PasswordResetContext {
        user_name,
        reset_link: &reset_link,
        site_name: host_display_name
    };

    send_templated_email(
        to_email,
        user_name,
        &settings.email.reset_password_subject,
        "templates/email/reset_password.hbs",
        "templates/email/reset_password_text.hbs",
        &context,
        settings,
    )
}



pub fn _send_account_confirmation_email_old(
    to_email: &str,
    user_name: &str,
    token: &str,
    host_display_name: &str, 
    settings: &Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build verification link
    let verification_link = format!(
        "{}/api/token/verify/{}",
        settings.email.base_url,
        token
    );


    // Initialize Handlebars
    let mut handlebars = Handlebars::new();

    // Load template from file
    handlebars.register_template_file(
        "verify_token",
        "templates/email/verify_token.hbs",
    )?;

    let context = AccountVerificationContext {
        user_name,
        verification_link: &verification_link,
        site_name: host_display_name
    };


    let html_body = handlebars.render("verify_token", &context)?;

    // Load template from file
    handlebars.register_template_file(
        "verify_token_text",
        "templates/email/verify_token_text.hbs",
    )?;

    let text_body = handlebars.render("verify_token_text", &context)?;

    // Construct the email
    let message_builder = Message::builder()
    
        .from(settings.email.smtp_from_email.parse()?)
        .to(Mailbox::new(Some(user_name.to_string()), to_email.parse()?))
        .subject(settings.email.verify_token_subject.clone())
        .multipart(
            lettre::message::MultiPart::alternative_plain_html(
                text_body,
                html_body,
            )
        )?;

    // SMTP transport
    let mailer = SmtpTransport::relay(settings.smtp.server.as_str())?
        .credentials((settings.smtp.username.as_str(), 
            settings.smtp.password.as_str()).into())
        .build();

    // Send the email
    mailer.send(&message_builder)?;
    Ok(())
}


use handlebars::Handlebars;

#[derive(Serialize)]
struct AccountVerificationContext<'a> {
    user_name: &'a str,
    verification_link: &'a str,
    pub site_name: &'a str,
}



fn _send_confirmation_email(to_email: &str, new_name: &str, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let confirmation_link = format!("https://revillagesociety.org/api/subscribe/confirm/{}", token);

    let new_email = Message::builder()
        .from("ReVillage Society <noreply@revillagesociety.org>".parse()?)
        .to(Mailbox::new(Some(new_name.to_string()), to_email.parse()?))
        .subject("Please confirm your subscription")
        .multipart(
            lettre::message::MultiPart::alternative_plain_html(
                format!("Click to confirm: {}", confirmation_link),
                format!(
                    "<p>Hello {},</p><p>Please confirm your subscription by clicking the link below:</p><p><a href=\"{}\">Confirm Subscription</a></p>",
                    new_name, confirmation_link
                ),
            )   
        )?;

    let mailer = SmtpTransport::relay("smtp.revillagesociety.org")?
        .credentials(("smtp-username", "smtp-password").into())
        .build();

    mailer.send(&new_email)?;
    Ok(())
}


#[cfg(test)]
mod integration_tests {
    use super::*;
    //use crate::{models::user_token::verify_user_token, test_support::db::setup_test_db};

    #[test]
    fn verify_handlebars_templates() {
     let mut handlebars = Handlebars::new();

    // Load template from file
    handlebars.register_template_file(
        "verify_token",
        "templates/email/verify_token.hbs",
    ).unwrap();

    let context = AccountVerificationContext {
        user_name: "Test User",
        verification_link: "SomeLink",
        site_name: "test"
    };


    let html_body = handlebars.render("verify_token", &context).unwrap();
    log::info!("Html Body: {:?}", html_body);


    // Load template from file
    handlebars.register_template_file(
        "verify_token_text",
        "templates/email/verify_token_text.hbs",
    ).unwrap();
    

    let text_body = handlebars.render("verify_token_text", &context).unwrap();
    log::info!("Text Body: {:?}", text_body);
    assert!(html_body.contains(&context.verification_link));
    assert!(html_body.contains(&context.user_name));
    
    assert!(text_body.contains(&context.verification_link));
    assert!(text_body.contains(&context.user_name));
    
    }
}

pub fn scope() -> Scope {
    web::scope("").service(subscribe)
        .service(confirm)
        .service(unsubscribe)
        .service(list_subscribers)
}