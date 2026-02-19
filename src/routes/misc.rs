use std::collections::HashMap;

use actix_session::Session;
use actix_web::Error;
use actix_web::{HttpResponse, get, web};
use chrono::NaiveDateTime;
use serde::Deserialize;
use crate::models::ticket::{self, NewTicket, Ticket};
use crate::models::users::{self, User};
use crate::routes::register;
use crate::types::method::Method;

#[cfg(debug_assertions)]
use crate::registration::RegisterQuery;
use crate::registration::{get_registrations, update_registration_user_id};
//use crate::models::ticket::{self, NewTicket, Ticket};

use crate::{app_state::AppState, errors::app_error::AppError};

use std::fs::OpenOptions;
use std::io::Write;





#[get("/ticketlist_DISABLED")]
async fn generate_ticket_ids(    
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // Generate a list of ticket IDs
    // retreave RSVP list form data.txt file
    let mut ticket_ids: HashMap<String, (User, Ticket)> = HashMap::new();
    // let file = OpenOptions::new()
    //     .read(true)
    //     .open("data.txt")
    //     .expect("Unable to open file");
    let mut conn = data.db_conn()?;

    //let reader = std::io::BufReader::new(file);
    let registrations = get_registrations(&mut conn)?;
    for rsvp in registrations {
      
        if rsvp.attend == false {
            log::info!("Skipping RSVP: {:?}", rsvp.clone());
            continue;
        }
        let event_id = rsvp.event_id.clone();   
        let rsvp_id = rsvp.id.clone();
        let user = users::find_or_create_user_by_email(&mut conn, rsvp).unwrap();
        update_registration_user_id(&mut conn, rsvp_id, user.id)
            .expect("Failed to update registration with user ID");
        // Check if the user already has a ticket


        let existing_ticket = ticket::find_ticket_by_user_id(&mut conn, user.id, event_id.clone());
        if existing_ticket.is_ok() {
            let existing_ticket = existing_ticket.unwrap();
           // log::info!("User already has a ticket: {:?}", existing_ticket.clone());

            ticket_ids.insert(existing_ticket.id.clone(), (user, existing_ticket));

            continue;
        } 
        let ticket_id = uuid::Uuid::new_v4().to_string();
        let ticket = NewTicket {
            id: ticket_id.clone(),
            user_id: user.id, //rsvp.name.clone(),
            //created_at: chrono::Utc::now().naive_utc(),
            checked_in: None,
            event_id: event_id,
            registration_id: Some(rsvp_id),
        };
        let ticket = ticket::create_ticket(&mut conn, ticket.clone()).expect("Failed to create ticket");
        log::info!("Created ticket: {:?}", ticket);
        //let ticket_url = format!("https://revillagesociety.org/api/ticket/{}", ticket_id);
        // Generate new QR code for each ticket
        //let qr_code = qrcode::QrCode::new(ticket_url.clone()).unwrap();
        //let image = qr_code.render::<Luma<u8>>().build();
        //let file_path = format!("qr_codes/{}.png", ticket_id);
        //image.save(&file_path).expect("Unable to save QR code");


        
        ticket_ids.insert(ticket_id.clone(), (user, ticket));
    }
    Ok(HttpResponse::Ok().json(ticket_ids))
}


#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]   
pub struct Twilio {

    #[allow(dead_code)]
    pub AccountSid: String,
        // Unique identifier of the account that generated the Debugger event. 
        
    #[allow(dead_code)]
    pub Sid	: String,   
        //Unique identifier of this Debugger event.
    #[allow(dead_code)]

        pub ParentAccountSid: Option<String>,
        //Unique identifier of the parent account. This parameter only exists if the above account is a subaccount.
    #[allow(dead_code)]

        pub Timestamp: NaiveDateTime,
        //Time of occurrence of the Debugger event.
    #[allow(dead_code)]

        pub Level	: String, 
        //Severity of the Debugger event. Possible values are Error and Warning.
    #[allow(dead_code)]

        pub PayloadType	: String, 
        //Type of the Debugger event. Possible values are DebuggerEvent and DebuggerEventError. application/json
    #[allow(dead_code)]
    pub Payload	: String, 
        // JSON data specific to the Debugger Event.
    
}




#[allow(dead_code)]
async fn twilio_handler(
    query: web::Query<Twilio>,
    _app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    //Receiving WebHook from twilio, its SHA256 hash of the body
    // to be decoded and verified
    let query = query.into_inner();
    log::info!("Twilio Handler form data: {:?}", query);
    // Return a response
    let return_string = r#"
    Thank you!    "#;

    Ok(HttpResponse::Ok().body(return_string))
}





//use validator::validate_credentials;

// TODO: create a new function to handle requests to the /api endpoint
// This function should read the contents of the file
// The requests come from an HTMLX post request
// With the following body: 

//curl 'http://localhost:8080/api/submit' \
// -X 'POST' \
// -H 'Content-Type: application/x-www-form-urlencoded' \
// -H 'Accept: */*' \
// -H 'Sec-Fetch-Site: same-origin' \
// -H 'Accept-Language: en-US,en;q=0.9' \
// -H 'Accept-Encoding: gzip, deflate' \
// -H 'Sec-Fetch-Mode: cors' \
// -H 'Origin: http://localhost:8080' \
// -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.4 Safari/605.1.15' \
// -H 'Referer: http://localhost:8080/register/?name=Your+NAme&email=email%40your.addr&phone=123-123-1231&attend=on&notification=on&source=friend&comments=add+comments' \
// -H 'Content-Length: 130' \
// -H 'Sec-Fetch-Dest: empty' \
// -H 'Connection: keep-alive' \
// -H 'HX-Request: true' \
// -H 'Priority: u=3, i' \
// -H 'HX-Target: response' \
// -H 'HX-Current-URL: http://localhost:8080/register/?name=Your+NAme&email=email%40your.addr&phone=123-123-1231&attend=on&notification=on&source=friend&comments=add+comments' \
// --data 'name=Your%20NAme&email=your%40email.addr&phone=123-123-1234&attend=on&notification=on&source=friend&comments=Additional%20Comments'

#[cfg(debug_assertions)]
async fn _register(
    query: web::Form<RegisterQuery>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    use crate::registration::{NewRegistration, create_registration};

    log::info!("Received form data: {:?}", query);

    let pool = &app_state.db_pool;
    let mut conn = pool.get().expect("Failed to get DB connection");
  
    let new_reg = NewRegistration::from(query.into_inner());
    log::info!("New Registration for Event: {:?}", new_reg.event_id);
    create_registration(&mut conn, new_reg.clone()).expect("Failed to create registration");

    // Open the file in append mode
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("data.txt")
        .expect("Unable to open file");
    //let query = new_reg;
    serde_json::to_writer(&mut file, &new_reg).expect("Unable to write JSON to file");
    writeln!(file).expect("Unable to write newline to file");

    // Return a response
    let return_string = r#"
        <h1 class="text-lg">Thank you for your submission!</h1>
        <p>We don't have an automated system yet, 
        so one of us will email or text you back soon.</p>
        <p><a class="button btn-lg button-primary" href="/">Go back</a></p>
        "#;


    Ok(HttpResponse::Ok().body(return_string))
}




#[get("/arrive/{ticket_id}")]
async fn process_ticket(
    ticket_id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let ticket_id = ticket_id.into_inner();
    log::info!("Scanned ticket ID: {}", ticket_id);

    // Return a response
    Ok(HttpResponse::Ok().body(format!("<a href=\"/api/checked-in/{}\">Check In</a>", ticket_id)))
}

#[get("/checked-in/{ticket_id}")]
async fn checkedin_ticket(
    ticket_id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let ticket_id = ticket_id.into_inner();
    log::info!("Checked in ticket ID: {}", ticket_id);


    // Return a response
    Ok(HttpResponse::Ok().body(format!("Ticket ID: {}", ticket_id)))
}


//#[get("/attend_event")]
async fn _attend_event(
    query: web::Query<Twilio>,
    _app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    //Receiving WebHook from twilio, its SHA256 hash of the body
    // to be decoded and verified
    let query = query.into_inner();
    log::info!("Received form data: {:?}", query);
    // Return a response
    let return_string = r#"
    Thank you!    "#;

    Ok(HttpResponse::Ok().body(return_string))
}



pub fn _require_login(session: &Session) -> Result<i32, Error> {
    if let Some(user_id) = session.get::<i32>("user_id")? {
        Ok(user_id)
    } else {
        Err(actix_web::error::ErrorUnauthorized("Not logged in"))
    }
}
