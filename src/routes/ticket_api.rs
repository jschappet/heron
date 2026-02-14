
use actix_web::{HttpResponse, Responder, Scope, delete, get, http::header::{ContentDisposition, DispositionParam, DispositionType}, post, put, web};
use image::{ImageFormat, Luma};
use serde::Deserialize;

use crate::{app_state::AppState, models::ticket::{NewTicket, assign_ticket_db, create_ticket, delete_ticket, get_ticket, get_tickets, get_tickets_for_event, update_ticket}};

// API Endpoints
#[post("")]
pub async fn create_ticket_api(
    data: web::Data<AppState>,
    new_ticket: web::Json<NewTicket>,
) -> impl Responder {
    let pool = &data.db_pool;
    let mut conn = pool.get().expect("Failed to get DB connection");
    match create_ticket(&mut conn, new_ticket.into_inner()) {
        Ok(ticket_new) => HttpResponse::Ok().json(ticket_new),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create ticket"),
    }
}


#[get("/event/{new_event_id}")]
pub async fn get_tickets_for_event_api(
    data: web::Data<AppState>,
    new_event_id: web::Path<String>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_tickets_for_event(conn, new_event_id.into_inner()) {
        Ok(ticket_data) => HttpResponse::Ok().json(ticket_data),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve tickets"),
    }
}

#[get("")]
pub async fn get_tickets_api(data: web::Data<AppState>) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_tickets(conn) {
        Ok(ticket_data) => HttpResponse::Ok().json(ticket_data),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve tickets"),
    }
}

#[get("/{ticket_id}")]
pub async fn get_ticket_api(
    data: web::Data<AppState>,
    ticket_id: web::Path<String>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_ticket(conn, ticket_id.into_inner()) {
        Ok(ticket_new) => HttpResponse::Ok().json(ticket_new),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve ticket"),
    }
}

#[put("/{ticket_id}")]
pub async fn update_ticket_api(
    data: web::Data<AppState>,
    ticket_id: web::Path<String>,
    updated_ticket: web::Json<NewTicket>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match update_ticket(conn, ticket_id.into_inner(), updated_ticket.into_inner()) {
        Ok(ticket_new) => HttpResponse::Ok().json(ticket_new),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update ticket"),
    }
}


// QR Code Image Generation
#[get("/{ticket_id}/qr")] 
pub async fn generate_qr_code(
    ticket_id: web::Path<String>,
) -> impl Responder {

    log::info!("Generating QR code for ticket ID: {}", ticket_id);
    let ticket_id = ticket_id.into_inner();
    let ticket_id = format!("https://revillagesociety.org/api/arrive/{}", ticket_id);
    // Generate QR code
    let qr_code = qrcode::QrCode::new(ticket_id.as_bytes()).unwrap();
    let image = qr_code.render::<Luma<u8>>().build();
    let mut buffer = std::io::Cursor::new(Vec::new());
    image.write_to(&mut buffer, ImageFormat::Png).unwrap();
    let buffer = buffer.into_inner();
    HttpResponse::Ok()
        .content_type("image/png")
        .append_header(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![DispositionParam::Filename(String::from("ticket_qr.png"))],
        })
        .body(buffer)
}


#[delete("/{ticket_id}")]
pub async fn delete_ticket_api(
    data: web::Data<AppState>,
    ticket_id: web::Path<String>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    let t_id = ticket_id.into_inner();
    match delete_ticket(conn, t_id.clone()) {
        Ok(_) => HttpResponse::Ok().body(format!("Ticket {:?} deleted", t_id)),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete ticket"),
    }
}

#[derive(Deserialize, Debug)]
pub struct AssignTicketRequest {
    reg_id: i32,
    user_id: i32,
    event_id: String,
}
 
#[post("/assign-ticket")]
async fn assign_ticket(
    data: web::Json<AssignTicketRequest>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let mut conn = app_data.db_pool.get().expect("Database connection failed");
    log::info!("Assigning ticket: {:?}", data);

    let evnt_id = data.0.event_id.clone();
    match assign_ticket_db(&mut conn, data.user_id, &evnt_id.as_str(), data.reg_id) {
        Ok(_) => HttpResponse::Ok().body("Ticket assigned"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to assign ticket"),
    }
    
}


pub fn scope() -> Scope {
    web::scope("")
        .service(create_ticket_api)
        .service(get_tickets_api)
        .service(get_ticket_api)
        .service(update_ticket_api)
        .service(delete_ticket_api)
        .service(generate_qr_code)
        .service(assign_ticket)
        .service(get_tickets_for_event_api)
}