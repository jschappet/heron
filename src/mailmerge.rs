use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use crate::{app_state::AppState };


// Define the structure for the response
#[derive(Serialize, Debug, QueryableByName)]
pub struct MailMergeRecipient {
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Text)]
    pub ticket: String,
}

fn get_recipients(
    new_event_id: String,
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<MailMergeRecipient>> {
    log::info!("Getting recipients for event ID: {}", new_event_id);

    let query = r#"
        SELECT 
            r.name AS name, 
            u.email AS email, 
            'https://revillagesociety.org/ticket/#' || t.id AS ticket
        FROM 
            registration r
        INNER JOIN 
            users u ON r.user_id = u.id
        INNER JOIN 
            ticket t ON r.user_id = t.user_id AND r.event_id = t.event_id
        WHERE 
            r.event_id = ?
    "#;

    sql_query(query)
        .bind::<Text, _>(new_event_id)
        .load::<MailMergeRecipient>(conn)
}

#[get("/mailmerge/{campaign_name}")]
pub async fn get_mailmerge_recipients(
    data: web::Data<AppState>,
    campaign_name: web::Path<String>,
) -> impl Responder {
    let mut conn = &mut data.db_pool.get().expect("Database connection failed");
    let campaign_name = campaign_name.into_inner();

    let recipients = get_recipients(campaign_name, &mut conn).expect("Failed to get recipients");

    HttpResponse::Ok().json(recipients)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_mailmerge_recipients);
}