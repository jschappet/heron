use actix_web::{HttpResponse, Responder, Scope, get, post, web};

#[get("/all")]
pub async fn get_ratings(data: web::Data<crate::AppState>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();

    log::info!("Fetching all ratings");
    match get_all_ratings(&mut conn, RatingType::Recipe) {
        Ok(ratings) => HttpResponse::Ok().json(ratings),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

use diesel::RunQueryDsl;
use serde::Deserialize;

use crate::models::rating_events::{
    NewRatingEvent, get_all_ratings, get_summary_map, rebuild_rating_summary,
};
use crate::types::RatingType;

#[derive(Deserialize)]
pub struct RatingInput {
    pub target_id: String,
    rating_type: String,
    pub rating: i32,
    pub review: Option<String>,
}

#[post("/save")]
pub async fn save_ratings(
    data: web::Data<crate::AppState>,
    //input_rating_type: web::Path<String>,
    in_rating: web::Json<RatingInput>,
    auth_context: AuthContext,
) -> impl Responder {
    let conn = &mut data.db_pool.get().unwrap();
    let r = in_rating.into_inner();
    let r_type = RatingType::from(r.rating_type.as_str());
    let evt = NewRatingEvent {
        rating_type: r_type.to_string(),
        target_id: r.target_id,
        user_id: Some(auth_context.user_id),
        rating: r.rating,
        review: r.review,
        rating_details: None,
    };

    // Insert event
    match diesel::insert_into(crate::schema::rating_events::table)
        .values(&evt)
        .execute(conn)
    {
        Ok(_) => {
            match rebuild_rating_summary(conn) {
                Ok(_) => HttpResponse::Ok().body("Rating saved"),
                Err(err) => {
                    eprintln!("Error rebuilding rating_summary: {:?}", err);
                    HttpResponse::InternalServerError().body("Failed to rebuild rating_summary")
                }
            }
            
        }
        Err(e) => {
            eprintln!("Error saving rating: {}", e);
            HttpResponse::InternalServerError().body("Error saving rating")
        }
    }
}

use crate::models::rating_events::get_all_summaries;
use crate::validator::AuthContext;

// ======================================================
// GET: /rating/summary/all
// Returns full aggregated table
// ======================================================

#[get("/summary/array")]
pub async fn get_summaries_array(data: web::Data<crate::AppState>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();

    match get_all_summaries(&mut conn) {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(err) => {
            eprintln!("Error loading summary table: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// ======================================================
// POST: /rating/rebuild-summary
// Runs full aggregation job
// ======================================================

#[post("/rebuild-summary")]
pub async fn rebuild_summary_route(data: web::Data<crate::AppState>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();

    match rebuild_rating_summary(&mut conn) {
        Ok(_) => HttpResponse::Ok().body("rating_summary successfully rebuilt"),
        Err(err) => {
            eprintln!("Error rebuilding rating_summary: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to rebuild rating_summary")
        }
    }
}

#[get("/summary/all")]
pub async fn get_summaries(data: web::Data<crate::AppState>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();

    match get_summary_map(&mut conn) {
        Ok(map) => HttpResponse::Ok().json(map),
        Err(err) => {
            eprintln!("Error loading rating summary: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}


pub fn scope() -> Scope {
    web::scope("").service(get_ratings)
        .service(save_ratings)
        .service(get_summaries)
        .service(rebuild_summary_route)

}