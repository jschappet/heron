use chrono::NaiveDateTime;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Bool, Integer, Nullable, Text, Timestamp},
};
use serde::Serialize;

#[derive(Debug, QueryableByName, Serialize)]
pub struct ContactMessage {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub host_id: i32,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Text)]
    pub message: String,
    #[diesel(sql_type = Timestamp)]
    pub created_at: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub read_at: Option<NaiveDateTime>,
}

#[derive(QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

pub fn record(
    conn: &mut SqliteConnection,
    host: i32,
    sender_name: &str,
    sender_email: &str,
    body: Option<&str>,
) -> QueryResult<()> {
    if let Some(body) = body.map(str::trim).filter(|body| !body.is_empty()) {
        diesel::sql_query(
            "INSERT INTO contact_messages (host_id, name, email, message) VALUES (?, ?, ?, ?)",
        )
        .bind::<Integer, _>(host)
        .bind::<Text, _>(sender_name.trim())
        .bind::<Text, _>(sender_email.trim())
        .bind::<Text, _>(body)
        .execute(conn)?;
    }
    Ok(())
}

pub fn count(conn: &mut SqliteConnection, host: i32, unread_only: bool) -> QueryResult<i64> {
    Ok(diesel::sql_query("SELECT COUNT(*) AS count FROM contact_messages WHERE host_id = ? AND (? = 0 OR read_at IS NULL)")
        .bind::<Integer,_>(host).bind::<Bool,_>(unread_only).get_result::<Count>(conn)?.count)
}

pub fn list(
    conn: &mut SqliteConnection,
    host: i32,
    unread_only: bool,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<ContactMessage>> {
    diesel::sql_query("SELECT * FROM contact_messages WHERE host_id = ? AND (? = 0 OR read_at IS NULL) ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?")
        .bind::<Integer,_>(host).bind::<Bool,_>(unread_only)
        .bind::<BigInt,_>(limit).bind::<BigInt,_>(offset).load(conn)
}

pub fn get(conn: &mut SqliteConnection, host: i32, message_id: i32) -> QueryResult<ContactMessage> {
    diesel::sql_query("SELECT * FROM contact_messages WHERE host_id = ? AND id = ?")
        .bind::<Integer, _>(host)
        .bind::<Integer, _>(message_id)
        .get_result(conn)
}

pub fn mark_read(
    conn: &mut SqliteConnection,
    host: i32,
    message_id: i32,
    read: bool,
) -> QueryResult<ContactMessage> {
    let updated = diesel::sql_query("UPDATE contact_messages SET read_at = CASE WHEN ? THEN COALESCE(read_at, CURRENT_TIMESTAMP) ELSE NULL END WHERE host_id = ? AND id = ?")
        .bind::<Bool,_>(read).bind::<Integer,_>(host).bind::<Integer,_>(message_id).execute(conn)?;
    if updated == 0 {
        return Err(diesel::result::Error::NotFound);
    }
    get(conn, host, message_id)
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct Reply {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub message_id: i32,
    #[diesel(sql_type = Integer)]
    pub user_id: i32,
    #[diesel(sql_type = Text)]
    pub request_id: String,
    #[diesel(sql_type = Text)]
    pub from_email: String,
    #[diesel(sql_type = Text)]
    pub subject: String,
    #[diesel(sql_type = Text)]
    pub body: String,
    #[diesel(sql_type = Text)]
    pub status: String,
    #[diesel(sql_type = Timestamp)]
    pub created_at: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub sent_at: Option<NaiveDateTime>,
}

#[derive(serde::Deserialize, Clone)]
pub struct ReplyInput {
    pub request_id: String,
    pub subject: String,
    pub body: String,
}

pub fn replies(conn: &mut SqliteConnection, host: i32, message_id: i32) -> QueryResult<Vec<Reply>> {
    get(conn, host, message_id)?;
    diesel::sql_query("SELECT * FROM contact_message_replies WHERE message_id = ? ORDER BY id ASC")
        .bind::<Integer, _>(message_id)
        .load(conn)
}

/// Persist an attempt before SMTP. A repeated request key never sends twice,
/// even if the client lost its response or the process stopped during delivery.
pub fn send_reply<F>(
    conn: &mut SqliteConnection,
    host: i32,
    message_id: i32,
    user: i32,
    sender: &str,
    input: &ReplyInput,
    send: F,
) -> Result<Reply, crate::errors::app_error::AppError>
where
    F: FnOnce(&ContactMessage, &ReplyInput) -> Result<(), String>,
{
    use crate::errors::app_error::AppError;
    let message = get(conn, host, message_id)?;
    if uuid::Uuid::parse_str(&input.request_id).is_err()
        || input.subject.trim().is_empty()
        || input.subject.len() > 180
        || input.subject.contains(['\r', '\n'])
        || input.body.trim().is_empty()
        || input.body.len() > 20_000
    {
        return Err(AppError::BadRequest("Provide a subject (up to 180 characters), a reply (up to 20,000 bytes), and a valid request ID.".into()));
    }
    let inserted = diesel::sql_query("INSERT INTO contact_message_replies (message_id, user_id, request_id, from_email, subject, body, status) VALUES (?, ?, ?, ?, ?, ?, 'sending') ON CONFLICT(message_id, request_id) DO NOTHING")
        .bind::<Integer,_>(message_id).bind::<Integer,_>(user).bind::<Text,_>(&input.request_id)
        .bind::<Text,_>(sender).bind::<Text,_>(&input.subject).bind::<Text,_>(&input.body).execute(conn)?;
    let lookup = |conn: &mut SqliteConnection| {
        diesel::sql_query(
            "SELECT * FROM contact_message_replies WHERE message_id = ? AND request_id = ?",
        )
        .bind::<Integer, _>(message_id)
        .bind::<Text, _>(&input.request_id)
        .get_result::<Reply>(conn)
    };
    if inserted == 0 {
        let existing = lookup(conn)?;
        if existing.subject != input.subject || existing.body != input.body {
            return Err(AppError::BadRequest(
                "This request ID belongs to a different reply. Reload before sending a new reply."
                    .into(),
            ));
        }
        return Ok(existing);
    }
    let status = match send(&message, input) {
        Ok(()) => "sent",
        Err(error) => {
            log::error!("Contact reply delivery failed for message {message_id}: {error}");
            "failed"
        }
    };
    diesel::sql_query("UPDATE contact_message_replies SET status = ?, sent_at = CASE WHEN ? = 'sent' THEN CURRENT_TIMESTAMP ELSE NULL END WHERE message_id = ? AND request_id = ?")
        .bind::<Text,_>(status).bind::<Text,_>(status).bind::<Integer,_>(message_id)
        .bind::<Text,_>(&input.request_id).execute(conn)?;
    Ok(lookup(conn)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn fixture() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        conn.batch_execute("PRAGMA foreign_keys = ON;
            CREATE TABLE hosts (id INTEGER PRIMARY KEY);
            INSERT INTO hosts VALUES (1), (3);
            CREATE TABLE users (id INTEGER PRIMARY KEY);
            INSERT INTO users VALUES (1);
            CREATE TABLE mailing_list_subscribers (host_id INTEGER, name TEXT, email TEXT, message TEXT, created_at TIMESTAMP);").unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-15-000000_contact_messages/up.sql"
        ))
        .unwrap();
        conn
    }

    #[test]
    fn migration_imports_only_nonempty_messages_and_preserves_subscribers() {
        let mut conn = fixture();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-15-000000_contact_messages/down.sql"
        ))
        .unwrap();
        conn.batch_execute(
            "INSERT INTO mailing_list_subscribers VALUES
            (1, 'Alice', 'a@example.com', 'Old message', '2026-01-01 00:00:00'),
            (3, 'Bob', 'b@example.com', '', '2026-01-02 00:00:00'),
            (3, 'Carol', 'c@example.com', NULL, '2026-01-03 00:00:00');",
        )
        .unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-15-000000_contact_messages/up.sql"
        ))
        .unwrap();
        assert_eq!(count(&mut conn, 1, true).unwrap(), 1);
        assert_eq!(count(&mut conn, 3, false).unwrap(), 0);
        let subscribers =
            diesel::sql_query("SELECT COUNT(*) AS count FROM mailing_list_subscribers")
                .get_result::<Count>(&mut conn)
                .unwrap();
        assert_eq!(subscribers.count, 3);
    }

    #[test]
    fn submissions_and_read_state_are_site_scoped() {
        let conn = &mut fixture();
        record(conn, 1, "Alice", "a@example.com", Some("First")).unwrap();
        record(conn, 1, "Alice", "a@example.com", Some("Second")).unwrap();
        record(conn, 3, "Alice", "a@example.com", Some("Private")).unwrap();
        record(conn, 1, "Alice", "a@example.com", Some("  ")).unwrap();
        assert_eq!(count(conn, 1, true).unwrap(), 2);
        let items = list(conn, 1, true, 30, 0).unwrap();
        assert_eq!(items[0].message, "Second");
        let id = items[0].id;
        assert!(get(conn, 3, id).is_err());
        assert!(mark_read(conn, 3, id, true).is_err());
        let first = mark_read(conn, 1, id, true).unwrap();
        assert_eq!(mark_read(conn, 1, id, true).unwrap().read_at, first.read_at);
        assert_eq!(count(conn, 1, true).unwrap(), 1);
        assert_eq!(count(conn, 1, false).unwrap(), 2);
        mark_read(conn, 1, id, false).unwrap();
        assert_eq!(count(conn, 1, true).unwrap(), 2);
    }

    #[test]
    fn reply_history_preserves_failures_and_prevents_duplicate_sends() {
        let user = 1;
        let conn = &mut fixture();
        record(conn, 1, "Alice", "a@example.com", Some("Hello")).unwrap();
        let id = list(conn, 1, false, 30, 0).unwrap()[0].id;
        let mut input = ReplyInput {
            request_id: uuid::Uuid::new_v4().to_string(),
            subject: "Re: Hello".into(),
            body: "Thanks".into(),
        };
        assert!(send_reply(
            conn,
            3,
            id,
            user,
            "admin@example.com",
            &input,
            |_, _| panic!("Cross-site send")
        )
        .is_err());
        let sent = send_reply(
            conn,
            1,
            id,
            user,
            "admin@example.com",
            &input,
            |message, _| {
                assert_eq!(message.email, "a@example.com");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(sent.status, "sent");
        assert!(sent.sent_at.is_some());
        let duplicate = send_reply(conn, 1, id, user, "admin@example.com", &input, |_, _| {
            panic!("Duplicate send")
        })
        .unwrap();
        assert_eq!(duplicate.id, sent.id);
        input.body = "Changed".into();
        assert!(send_reply(
            conn,
            1,
            id,
            user,
            "admin@example.com",
            &input,
            |_, _| panic!("Conflicting key")
        )
        .is_err());
        input.request_id = uuid::Uuid::new_v4().to_string();
        let failed = send_reply(conn, 1, id, user, "admin@example.com", &input, |_, _| {
            Err("Mock SMTP failure".into())
        })
        .unwrap();
        assert_eq!(failed.status, "failed");
        assert!(failed.sent_at.is_none());
        assert_eq!(replies(conn, 1, id).unwrap().len(), 2);
        assert!(replies(conn, 3, id).is_err());
        input.subject = "Injected\r\nBcc: other@example.com".into();
        input.request_id = uuid::Uuid::new_v4().to_string();
        assert!(send_reply(
            conn,
            1,
            id,
            user,
            "admin@example.com",
            &input,
            |_, _| panic!("Invalid header")
        )
        .is_err());
    }
}
