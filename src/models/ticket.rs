
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use chrono::{NaiveDateTime, Utc};

use uuid::Uuid;
use crate::users::User;
use crate::{registration, users};
use crate::registration::Registration;
use crate::schema::ticket::dsl::ticket;
use crate::schema::ticket::*;
use serde::{Deserialize, Serialize};

// Ticket Model
#[derive(Debug, Queryable, Selectable, Clone, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ticket)]
pub struct Ticket {
    pub id: String,
    pub user_id: i32,
    pub event_id: String,
    pub registration_id: Option<i32>,
    pub checked_in: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug,  Serialize, Deserialize)]
pub struct TicketView {
    pub id: String,
    pub user: User,
    pub event_id: String,
    pub registration: Registration,
    pub checked_in: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}


#[derive(Debug, Queryable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::ticket)]
pub struct NewTicket {
    pub id: String,
    pub user_id: i32,
    pub event_id: String,
    pub registration_id: Option<i32>,
    pub checked_in: Option<NaiveDateTime>,
}

// Create a new ticket
pub fn create_ticket(
    conn: &mut SqliteConnection,
    new_ticket: NewTicket,
) -> QueryResult<Ticket> {
    let now: NaiveDateTime = Utc::now().naive_utc();
    diesel::insert_into(ticket)
        .values((
            id.eq(new_ticket.id),
            user_id.eq(new_ticket.user_id),
            event_id.eq(new_ticket.event_id),
            registration_id.eq(Some(0)),
            checked_in.eq(new_ticket.checked_in),
            created_at.eq(now),
        ))
        .execute(conn)?;

    ticket.order(id.desc()).first::<Ticket>(conn)
}

// Retrieve all tickets
pub fn get_tickets(conn: &mut SqliteConnection) -> QueryResult<Vec<Ticket>> {
    ticket.load::<Ticket>(conn)
}

// Retrieve all tickets for event
pub fn get_tickets_for_event(
    conn: &mut SqliteConnection,
    new_event_id: String,
) -> QueryResult<Vec<TicketView>> {
    
    let tickets_list = ticket
        .filter(event_id.eq(new_event_id))
        .load::<Ticket>(conn);
    // Convert AllTicket to TicketView
    let mut ticket_view_list: Vec<TicketView> = vec![];
    if tickets_list.is_ok() {
        for other_ticket in tickets_list.unwrap() {
            let ticket_view = convert_ticket_to_view(conn, other_ticket);
            ticket_view_list.push(ticket_view);
        }
    }   
    Ok(ticket_view_list)
}

pub fn convert_ticket_to_view(
       conn: &mut SqliteConnection,
    in_ticket: Ticket,
) -> TicketView {

    let user = users::get_user_by_id(conn, in_ticket.user_id).unwrap();
    let registration = registration::get_registration(conn, in_ticket.registration_id.unwrap()).unwrap();
    TicketView {
        id: in_ticket.id,
        user,
        event_id: in_ticket.event_id,
        registration,
        checked_in: in_ticket.checked_in,
        created_at: in_ticket.created_at,
    }
}


// Retrieve a specific ticket by user ID and event ID
pub fn find_ticket_by_user_id(
    conn: &mut SqliteConnection,
    new_user_id: i32,
    new_event_id: String,
) -> QueryResult<Ticket> {
    ticket
        .filter(user_id.eq(new_user_id))
        .filter(event_id.eq(new_event_id))
        .first::<Ticket>(conn)
}

// Retrieve a specific ticket by ID
pub fn get_ticket(conn: &mut SqliteConnection, ticket_id: String) -> QueryResult<Ticket> {
    ticket.find(ticket_id).first::<Ticket>(conn)
}

// Update a ticket
pub fn update_ticket(
    conn: &mut SqliteConnection,
    ticket_id: String,
    updated_ticket: NewTicket,
) -> QueryResult<Ticket> {
    diesel::update(ticket.find(ticket_id.clone()))
        .set((
            user_id.eq(updated_ticket.user_id),
            event_id.eq(updated_ticket.event_id),
            checked_in.eq(updated_ticket.checked_in),
            registration_id.eq(updated_ticket.registration_id),
        ))
        .execute(conn)?;

    ticket.find(ticket_id).first::<Ticket>(conn)
}


pub fn assign_ticket_db(
    conn: &mut SqliteConnection,
    usr_id: i32,
    evt_id: &str,
    reg_id: i32,
) -> QueryResult<Ticket> {
 let new_id = Uuid::new_v4().to_string();

    diesel::insert_into(ticket)
        .values((
            id.eq(new_id),
            user_id.eq(usr_id),
            event_id.eq(evt_id),
            registration_id.eq(reg_id),
            
            created_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)?;
     ticket.order(id.desc()).first::<Ticket>(conn)
    
    
}

// Delete a ticket
pub fn delete_ticket(conn: &mut SqliteConnection, ticket_id: String) -> QueryResult<usize> {
    diesel::delete(ticket.find(ticket_id)).execute(conn)
}
