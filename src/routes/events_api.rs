
// --- Actix Web API Endpoints ---

use actix_web::{HttpResponse, Responder, Scope, get, delete, post, put, web::{self}};
use serde::Serialize;
use crate::routes::register;
use crate::types::method::Method;
use crate::{app_state::AppState, db::{PendingRegistration, load_pending_registrations}, models::events::{NewEvent, create_event, delete_event, get_event, get_events, update_event}};


pub async fn create_event_api(
    data: web::Data<AppState>,
    new_event: web::Json<NewEvent>,
) -> impl Responder {
    let pool = &data.db_pool;
    let mut conn = pool.get().expect("Failed to get DB connection");
    match create_event(&mut conn, new_event.into_inner()) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create event"),
    }
}


pub async fn get_events_api(data: web::Data<AppState>) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_events(conn) {
        Ok(events_data) => HttpResponse::Ok().json(events_data),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve events"),
    }
}


pub async fn get_event_api(
    data: web::Data<AppState>,
    event_id: web::Path<String>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_event(conn, event_id.into_inner()) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve event"),
    }
}


pub async fn update_event_api(
    data: web::Data<AppState>,
    event_id: web::Path<String>,
    updated_event: web::Json<NewEvent>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match update_event(conn, event_id.into_inner(), updated_event.into_inner()) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update event"),
    }
}


pub async fn delete_event_api(
    data: web::Data<AppState>,
    event_id: web::Path<String>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    let e_id = event_id.into_inner();
    match delete_event(conn, e_id.clone()) {
        Ok(_) => HttpResponse::Ok().body(format!("Event {:?} deleted", e_id)),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete event"),
    }
}




#[derive(Serialize)]
struct PendingContext {
    event_id: String,
    registrations: Vec<PendingRegistration>,
}


async fn get_pending_registrations_html(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let event_id = path.into_inner();
    let mut conn = data.db_pool.get().expect("Failed to get DB connection");
    let tmpl = data.hb.clone();
    let new_event_id = event_id.clone();
    let regs_result = 
        web::block(move || load_pending_registrations(&mut conn, &new_event_id)).await;

    match regs_result.unwrap() {
        Ok(registrations) => {
            let context = PendingContext {
                event_id,
                registrations,
            };
            let rendered = tmpl.render("pending_registrations", &context);
            match rendered {
                Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
                Err(err) => {
                    eprintln!("Handlebars render error: {:?}", err);
                    HttpResponse::InternalServerError().body("Template error")
                }
            }
        }
        Err(err) => {
            eprintln!("Error fetching registrations: {:?}", err);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}


pub fn admin_scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
    // Events API Registration

// Create a new event
.service(register(
    "create_event",
    Method::POST,
    &full_path,
    "events",
    create_event_api,
    crate::types::MemberRole::Admin,
))

// List all events
.service(register(
    "get_events",
    Method::GET,
    &full_path,
    "events",
    get_events_api,
    crate::types::MemberRole::Admin,
))

// Get single event
.service(register(
    "get_event",
    Method::GET,
    &full_path,
    "event/{event_id}",
    get_event_api,
    crate::types::MemberRole::Admin,
))

// Update event
.service(register(
    "update_event",
    Method::PUT,
    &full_path,
    "event/{event_id}",
    update_event_api,
    crate::types::MemberRole::Admin,
))

// Delete event
.service(register(
    "delete_event",
    Method::DELETE,
    &full_path,
    "event/{event_id}",
    delete_event_api,
    crate::types::MemberRole::Admin,
))

// Pending registrations for event (HTML)
.service(register(
    "get_pending_registrations",
    Method::GET,
    &full_path,
    "events/{event_id}/pending-registrations",
    get_pending_registrations_html,
    crate::types::MemberRole::Admin,
))

}

// .service(create_event_api)
//         .service(get_events_api)
//         .service(get_event_api)
//         .service(update_event_api)
//         .service(delete_event_api)
//         .service(get_pending_registrations_html)