use crate::{
    app_state::AppState,
    errors::app_error::AppError,
    middleware::host::HostContext,
    routes::{register, RoutePath},
    services::contact_messages as messages,
    types::{method::Method, MemberRole},
    validator::{require_role_for_host, AuthContext},
};
use actix_web::{web, HttpResponse, Scope};
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};
use serde::Deserialize;
use serde_json::json;

fn authorize(auth: &AuthContext, host: &HostContext) -> Result<(), AppError> {
    require_role_for_host(auth, host.0.id, &[MemberRole::Admin])?;
    if host.0.id <= 0 {
        return Err(AppError::BadRequest("Unknown site.".into()));
    }
    Ok(())
}

#[derive(Deserialize)]
struct ListQuery {
    unread: Option<bool>,
    offset: Option<i64>,
}

async fn list(
    auth: AuthContext,
    host: HostContext,
    data: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    authorize(&auth, &host)?;
    let site = host.0.id;
    let offset = query.offset.unwrap_or(0).max(0);
    let unread = query.unread.unwrap_or(false);
    let result = web::block(move || -> Result<_, AppError> {
        let mut conn = data.db_pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(json!({"messages": messages::list(&mut conn, site, unread, 30, offset)?,
            "total": messages::count(&mut conn, site, unread)?,
            "unread_count": messages::count(&mut conn, site, true)?, "limit": 30, "offset": offset}))
    }).await.map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(result))
}

async fn count(
    auth: AuthContext,
    host: HostContext,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    authorize(&auth, &host)?;
    let site = host.0.id;
    let count = web::block(move || -> Result<_, AppError> {
        let mut conn = data
            .db_pool
            .get()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(messages::count(&mut conn, site, true)?)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(json!({"count": count})))
}

async fn detail(
    auth: AuthContext,
    host: HostContext,
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    authorize(&auth, &host)?;
    let site = host.0.id;
    let reply_from = sender(&data.settings, &host.0.display_name)
        .ok()
        .map(|value| value.to_string());
    let result = web::block(move || -> Result<_, AppError> {
        let mut conn = data
            .db_pool
            .get()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(json!({"message": messages::get(&mut conn, site, *id)?,
            "replies": messages::replies(&mut conn, site, *id)?, "reply_from": reply_from}))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(result))
}

#[derive(Deserialize)]
struct ReadInput {
    read: bool,
}

async fn mark_read(
    auth: AuthContext,
    host: HostContext,
    data: web::Data<AppState>,
    id: web::Path<i32>,
    input: web::Json<ReadInput>,
) -> Result<HttpResponse, AppError> {
    authorize(&auth, &host)?;
    let site = host.0.id;
    let result = web::block(move || -> Result<_, AppError> {
        let mut conn = data
            .db_pool
            .get()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let message = messages::mark_read(&mut conn, site, *id, input.read)?;
        Ok(json!({"message": message, "unread_count": messages::count(&mut conn, site, true)?}))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(HttpResponse::Ok().json(result))
}

fn sender(settings: &crate::settings::Settings, site_name: &str) -> Result<Mailbox, AppError> {
    let configured = if settings.email.smtp_from_email.trim().is_empty() {
        &settings.smtp.username
    } else {
        &settings.email.smtp_from_email
    };
    let mailbox: Mailbox = configured.parse().map_err(|_| {
        AppError::Internal("Configure a valid email.smtp_from_email before replying.".into())
    })?;
    Ok(Mailbox::new(Some(site_name.to_string()), mailbox.email))
}

async fn reply(
    auth: AuthContext,
    host: HostContext,
    data: web::Data<AppState>,
    id: web::Path<i32>,
    input: web::Json<messages::ReplyInput>,
) -> Result<HttpResponse, AppError> {
    authorize(&auth, &host)?;
    let site = host.0.id;
    let from = sender(&data.settings, &host.0.display_name)?;
    let result = web::block(move || -> Result<_, AppError> {
        let mut conn = data
            .db_pool
            .get()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        messages::send_reply(
            &mut conn,
            site,
            *id,
            auth.user_id,
            &from.to_string(),
            &input,
            |contact, reply| {
                let email = Message::builder()
                    .from(from.clone())
                    .reply_to(from)
                    .to(Mailbox::new(
                        Some(contact.name.clone()),
                        contact
                            .email
                            .parse()
                            .map_err(|e| format!("Invalid recipient: {e}"))?,
                    ))
                    .subject(&reply.subject)
                    .body(reply.body.clone())
                    .map_err(|e| e.to_string())?;
                let mailer = SmtpTransport::relay(&data.settings.smtp.server)
                    .map_err(|e| e.to_string())?
                    .port(data.settings.smtp.port)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .credentials(
                        (
                            data.settings.smtp.username.clone(),
                            data.settings.smtp.password.clone(),
                        )
                            .into(),
                    )
                    .build();
                mailer.send(&email).map_err(|e| e.to_string())?;
                Ok(())
            },
        )
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(HttpResponse::Ok().json(result))
}

pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register(
            "contact_messages",
            Method::GET,
            path.as_str(),
            "",
            list,
            MemberRole::Admin,
        ))
        .service(register(
            "contact_message_count",
            Method::GET,
            path.as_str(),
            "count",
            count,
            MemberRole::Admin,
        ))
        .service(register(
            "contact_message",
            Method::GET,
            path.as_str(),
            "{id}",
            detail,
            MemberRole::Admin,
        ))
        .service(register(
            "contact_message_read",
            Method::PUT,
            path.as_str(),
            "{id}/read",
            mark_read,
            MemberRole::Admin,
        ))
        .service(register(
            "contact_message_reply",
            Method::POST,
            path.as_str(),
            "{id}/replies",
            reply,
            MemberRole::Admin,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{middleware::host::HostInfo, validator::MembershipContext};
    #[test]
    fn inbox_requires_admin_for_the_resolved_site() {
        let host = HostContext(std::rc::Rc::new(HostInfo {
            id: 3,
            slug: "test".into(),
            host_name: "test.localhost".into(),
            display_name: "Test".into(),
            base_url: "http://test.localhost".into(),
        }));
        let mut auth = AuthContext {
            user_id: 1,
            memberships: vec![],
        };
        assert!(authorize(&auth, &host).is_err());
        auth.memberships.push(MembershipContext {
            host_id: 1,
            role: MemberRole::Admin,
        });
        assert!(authorize(&auth, &host).is_err());
        auth.memberships.push(MembershipContext {
            host_id: 3,
            role: MemberRole::Admin,
        });
        assert!(authorize(&auth, &host).is_ok());
    }
}
