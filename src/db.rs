
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{ sql_query, sql_types::Text, QueryResult, QueryableByName, RunQueryDsl, SqliteConnection};

use diesel::sql_types::{Integer,  Nullable, Timestamp};
use serde::Serialize;

/// Alias for a pooled connection
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;
pub type DbPool = Pool<ConnectionManager<SqliteConnection>> ;


use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::errors::app_error::AppError;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migrations(conn: &mut SqliteConnection) -> Result<(), AppError> {
    // SQLite needs this for foreign keys to actually work
    diesel::sql_query("PRAGMA foreign_keys = ON;")
        .execute(conn)?;

    // Run all pending migrations
    conn.run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|e| {
            log::error!("Database migrations failed: {:?}", e);
            AppError::Internal("Database migrations failed".to_string())
        })
}



#[derive(QueryableByName, Serialize, Debug)]
pub struct PendingRegistration {
    #[diesel(sql_type = Integer)]
    pub registration_id: i32,

    #[diesel(sql_type = Text)]
    pub event_id: String,
    
    #[diesel(sql_type = Integer)]
    pub user_id: i32,
    
    #[diesel(sql_type = Text)]
    pub person_name: String,
    
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Text)]
    pub phone: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub source: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub comments: Option<String>,
    #[diesel(sql_type = Timestamp)]
    pub event_created_at: chrono::NaiveDateTime,
    #[diesel(sql_type = Text)]
    pub username: String,
}



pub fn load_pending_registrations(
    conn: &mut SqliteConnection,
    event_id: &str,
) -> QueryResult<Vec<PendingRegistration>> {
    sql_query(
        r#"
        SELECT 
            r.id registration_id, 
            r.event_id event_id,
            u.id user_id,
            u.username, 
            r.name person_name,
            r.email as email,
            r.source as source,
            r.phone as phone,
            r.comments as comments,
            e.created_at event_created_at
        FROM registration r
        JOIN users u ON u.id = r.user_id
        JOIN events e ON e.id = r.event_id
        WHERE r.event_id = ? AND r.attend = 1
        AND NOT EXISTS (
            SELECT 1 FROM ticket t
            WHERE t.user_id = r.user_id AND t.event_id = r.event_id
        )
        "#
    )
    .bind::<Text, _>(event_id)
    .load::<PendingRegistration>(conn)
}
