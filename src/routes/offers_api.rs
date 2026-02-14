use actix_web::{HttpResponse, Scope, delete, get, post, put, web};
use serde::Deserialize;

use crate::errors::app_error::AppError;
use crate::models::contribution:: WantsToContributeInput;
use crate::models::offers::{
    OfferChangeset, create_offer, delete_offer, get_offer, get_offers, get_user_offers,
    update_offer,
};

use crate::app_state::AppState;
use crate::schema::{contribution_events, wants_to_contribute};
use crate::validator::AuthContext;
use diesel::RunQueryDsl;

#[post("")]
pub async fn create_offer_api(
    auth_context: AuthContext,
    data: web::Data<AppState>,
    new_offer: web::Json<OfferChangeset>,
) -> Result<HttpResponse, AppError> {
    let new_offer = OfferChangeset {
        user_id: Some(auth_context.user_id),
        ..new_offer.into_inner()
    }; // Ensure the offer is linked to the authenticated user
    let mut conn = data.db_conn()?;
    let offer_obj = create_offer(&mut conn, new_offer)?;
    Ok(HttpResponse::Ok().json(offer_obj))
}

#[get("/{id}")]
pub async fn get_offer_api(
    data: web::Data<AppState>,
    offer_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let offer_obj = get_offer(&mut conn, offer_id.into_inner())?;
    Ok(HttpResponse::Ok().json(offer_obj))
}

#[get("")]
pub async fn get_offers_api(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let offer_obj = get_offers(&mut conn)?;
    Ok(HttpResponse::Ok().json(offer_obj))
}

#[get("/user/{input_uid}/offers")]
pub async fn get_user_offers_api(
    data: web::Data<AppState>,
    input_uid: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let user_offers = get_user_offers(&mut conn, input_uid.into_inner())?;
    Ok(HttpResponse::Ok().json(user_offers))
}

#[put("/{id}")]
pub async fn update_offer_api(
    data: web::Data<AppState>,
    offer_id: web::Path<i32>,
    updated_offer: web::Json<OfferChangeset>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    log::info!("Updating offer ID: {:?}", updated_offer);
    let offer_obj = update_offer(&mut conn, offer_id.into_inner(), updated_offer.into_inner())?;
    Ok(HttpResponse::Ok().json(offer_obj))
}

#[delete("/{id}")]
pub async fn delete_offer_api(
    data: web::Data<AppState>,
    offer_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    delete_offer(&mut conn, offer_id.into_inner())?;
    Ok(HttpResponse::Ok().body("Offer deleted"))
}

#[derive(Deserialize)]
pub struct WantsToHelpData {
    pub offer_id: i32,
    pub who: Option<String>,
    pub how_helping: Option<String>,
    pub availability_days: Option<String>,
    pub availability_times: Option<String>,
    pub notes: Option<String>,
}

#[post("/wants_to_help")]
pub async fn create_wants_to_contribute(
    auth_context: AuthContext,
    data: web::Data<AppState>,
    payload: web::Json<WantsToHelpData>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let payload = payload.into_inner();
    let input = WantsToContributeInput {
        offer_id: payload.offer_id,
        helper_user_id: auth_context.user_id,
        who: payload.who,
        how_helping: payload.how_helping,
        availability_days: payload.availability_days,
        availability_times: payload.availability_times,
        notes: payload.notes,
        
    };

    let _ = diesel::insert_into(wants_to_contribute::table)
        .values(&input)
        .execute(&mut conn)?;

    Ok(HttpResponse::Ok().finish())
}
/* 
#[post("/help_events")]
pub async fn create_contribute_event(
    auth_context: AuthContext,
    data: web::Data<AppState>,
    payload: web::Json<ContribtionEventInput>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;

    let input = ContribtionEventInput {
        helper_user_id: auth_context.user_id,
        ..payload.into_inner()
    };

    let _ = diesel::insert_into(contribution_events::table)
        .values(&input)
        .execute(&mut conn);

    Ok(HttpResponse::Ok().finish())
}
 */

pub fn scope() -> Scope {
    web::scope("")
        .service(create_offer_api)
        .service(get_offer_api)
        .service(get_user_offers_api)
        .service(update_offer_api)
        .service(delete_offer_api)
        .service(get_offers_api)
        .service(create_wants_to_contribute)
        //.service(create_contribute_event)
}
