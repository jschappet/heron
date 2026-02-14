use diesel::prelude::*;
// use diesel::query_builder::Query;
use diesel::sqlite::SqliteConnection;
use chrono::{NaiveDateTime, Utc};
use crate::{app_state::AppState, schema::registration::dsl::registration};
use crate::schema::registration::*;
use serde::{Deserialize, Serialize};


use actix_web::{delete, get, post, web, HttpResponse, Responder};



#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct RegisterQuery {
    pub name: String,
    pub event_id: String,
    pub email: String,
    pub phone: String,
    pub attend: Option<String>,
    pub notification: Option<String>,
    pub source: Option<String>,
    pub comments: Option<String>,
}

// Registration Model
#[derive(Debug, Queryable, Selectable, Insertable, Clone, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::registration)]
pub struct Registration {
    pub id: i32,
    pub event_id: String,
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub attend: bool,
    pub notification: bool,
    pub source: Option<String>,
    pub comments: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

// NewRegistration from RegisterQuery
impl  From<RegisterQuery> for NewRegistration {
    fn from(query: RegisterQuery) -> Self {
        NewRegistration {
            event_id: query.event_id, 
            user_id: 0, // Placeholder, should be set when creating
            name: query.name,
            email: query.email,
            phone: query.phone,
            attend: query.attend.is_some(),
            notification: query.notification.is_some(),
            source: query.source,
            comments: query.comments,
        }
    }
}



#[derive(Debug, Queryable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::registration)]
pub struct NewRegistration {
    pub event_id: String,
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub attend: bool,
    pub notification: bool,
    pub source: Option<String>,
    pub comments: Option<String>,
}



// Create a new registration
pub fn create_registration(
    conn: &mut SqliteConnection,
    new_registration: NewRegistration,
) -> QueryResult<Registration> {
    let now: NaiveDateTime = Utc::now().naive_utc();
    diesel::insert_into(registration)
        .values((
            event_id.eq(new_registration.event_id),
            user_id.eq(new_registration.user_id),
            name.eq(new_registration.name),
            email.eq(new_registration.email),
            phone.eq(new_registration.phone),
            attend.eq(new_registration.attend),
            notification.eq(new_registration.notification),
            source.eq(new_registration.source),
            comments.eq(new_registration.comments),
            created_at.eq(now),
        ))
        .execute(conn)?;

    registration.order(id.desc()).first::<Registration>(conn)
}


// For a givent registration, update the user_id
pub fn update_registration_user_id(
    conn: &mut SqliteConnection,
    registration_id: i32,
    new_user_id: i32,
) -> QueryResult<Registration> {
    diesel::update(registration.find(registration_id))
        .set(user_id.eq(new_user_id))
        .execute(conn)?;

    registration.find(registration_id).first::<Registration>(conn)
}

// Retrieve all registrations
pub fn get_registrations(conn: &mut SqliteConnection) -> QueryResult<Vec<Registration>> {
    // ORDER By created_at descending
    registration
        .order(created_at.desc())
        .load::<Registration>(conn)
    
}

//get registrations for a specific user
// Order by created_at descending
// Limit 1  
pub fn _get_registration_for_user(
    conn: &mut SqliteConnection, new_user_id: i32) ->  QueryResult<Registration> {
    registration
        .filter(user_id.eq(new_user_id))
        .order(created_at.desc())
        .first::<Registration>(conn)
}

// Retrieve a specific registration by ID
pub fn get_registration(conn: &mut SqliteConnection, registration_id: i32) -> QueryResult<Registration> {
    registration.find(registration_id).first::<Registration>(conn)
}



// Update a registration

/* 
pub fn update_registration(
    conn: &mut SqliteConnection,
    registration_id: i32,
    updated_registration: NewRegistration,
) -> QueryResult<Registration> {
    diesel::update(registration.find(registration_id))
        .set((
            event_id.eq(updated_registration.event_id),
            user_id.eq(updated_registration.user_id),
            name.eq(updated_registration.name),
            email.eq(updated_registration.email),
            phone.eq(updated_registration.phone),
            attend.eq(updated_registration.attend),
            notification.eq(updated_registration.notification),
            source.eq(updated_registration.source),
            comments.eq(updated_registration.comments),
        ))
        .execute(conn)?;

    registration.find(registration_id).first::<Registration>(conn)
}
 */

// Delete a registration
pub fn delete_registration(conn: &mut SqliteConnection, registration_id: i32) -> QueryResult<usize> {
    diesel::delete(registration.find(registration_id)).execute(conn)
}

//use actix_web::{ post, web, HttpResponse, Responder};

// API Endpoints
#[post("/registrations")]
pub async fn create_registration_api(
    data: web::Data<AppState>,
    new_registration: web::Json<NewRegistration>,
) -> impl Responder {
    let pool = &data.db_pool;
    let mut conn = pool.get().expect("Failed to get DB connection");
    match create_registration(&mut conn, new_registration.into_inner()) {
        Ok(registration_new) => HttpResponse::Ok().json(registration_new),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create registration"),
    }
}

#[get("/registrations")]
pub async fn get_registrations_api(data: web::Data<AppState>) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_registrations(conn) {
        Ok(registration_data) => HttpResponse::Ok().json(registration_data),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve registrations"),
    }
}

#[get("/registration/{registration_id}")]
pub async fn get_registration_api(
    data: web::Data<AppState>,
    registration_id: web::Path<i32>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match get_registration(conn, registration_id.into_inner()) {
        Ok(registration_new) => HttpResponse::Ok().json(registration_new),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve registration"),
    }
}

/* 
#[put("/registration/{registration_id}")]
pub async fn update_registration_api(
    data: web::Data<AppState>,
    registration_id: web::Path<i32>,
    updated_registration: web::Json<NewRegistration>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    match update_registration(conn, registration_id.into_inner(), updated_registration.into_inner()) {
        Ok(registration_new) => HttpResponse::Ok().json(registration_new),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update registration"),
    }
}  */

#[delete("/registration/{registration_id}")]
pub async fn delete_registration_api(
    data: web::Data<AppState>,
    registration_id: web::Path<i32>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("Database connection failed");
    let reg_id = registration_id.into_inner();
    match delete_registration(conn, reg_id) {
        Ok(_) => HttpResponse::Ok().body(format!("Registration {:?} deleted", reg_id)),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete registration"),
    }
}




// Configure API routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_registration_api)
        .service(get_registrations_api)
        .service(get_registration_api)
     //   .service(update_registration_api)
        .service(delete_registration_api)
        ;
}

