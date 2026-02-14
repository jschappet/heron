use std::collections::HashMap;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono:: Utc;

use crate::{
    schema::{rating_events, rating_summary},
    types::RatingType,
};

// ==========================================
// Rating Event (append-only)
// ==========================================

#[derive(Debug, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = rating_events)]
pub struct RatingEvent {
    pub id: i32,
    pub rating_type: String,
    pub target_id: String,
    pub user_id: Option<i32>,
    pub rating: i32,
    pub review: Option<String>,
    pub rating_details: Option<String>,
    pub created_at: String, // SQLite stores text timestamps
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = rating_events)]
pub struct NewRatingEvent {
    pub rating_type: String,
    pub target_id: String,
    pub user_id: Option<i32>,
    pub rating: i32,
    pub review: Option<String>,
    pub rating_details: Option<String>,
}

// ==========================================
// Rating Summary (aggregated)
// ==========================================

#[derive(Debug, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = rating_summary)]
#[diesel(primary_key(rating_type, target_id))]
pub struct RatingSummary {
    pub rating_type: String,
    pub target_id: String,
    pub rating_sum: i32,
    pub rating_count: i32,
    pub average_rating: f32,
    pub last_updated: String,
}

#[derive(Debug, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = rating_summary)]
pub struct NewRatingSummary {
    pub rating_type: String,
    pub target_id: String,
    pub rating_sum: i32,
    pub rating_count: i32,
    pub average_rating: f32,
    pub last_updated: String,
}

// ==========================================
// CRUD Operations for rating_events
// ==========================================

pub fn _create_rating_event(
    conn: &mut SqliteConnection,
    evt: NewRatingEvent,
) -> QueryResult<usize> {
    diesel::insert_into(rating_events::table)
        .values(evt)
        .execute(conn)
}

pub fn get_all_ratings(
    conn: &mut SqliteConnection,
    rating_type_filter: RatingType,
) -> QueryResult<Vec<RatingEvent>> {
    log::info!("Getting all ratings for type: {:?}", rating_type_filter);

    let results = rating_events::table
        .filter(rating_events::rating_type.eq(rating_type_filter.as_str()))
        .order(rating_events::created_at.desc())
        .load::<RatingEvent>(conn);

    log::info!("Fetched all ratings: {:?}", results);
    results
}

pub fn _get_events_for_target(
    conn: &mut SqliteConnection,
    r_type: &str,
    t_id: &str,
) -> QueryResult<Vec<RatingEvent>> {
    use crate::schema::rating_events::dsl::*;

    rating_events
        .filter(rating_type.eq(r_type))
        .filter(target_id.eq(t_id))
        .order(created_at.desc())
        .load::<RatingEvent>(conn)
}

pub fn _delete_events_for_target(
    conn: &mut SqliteConnection,
    r_type: &str,
    t_id: &str,
) -> QueryResult<usize> {
    use crate::schema::rating_events::dsl::*;

    diesel::delete(
        rating_events
            .filter(rating_type.eq(r_type))
            .filter(target_id.eq(t_id)),
    )
    .execute(conn)
}

// ==========================================
// Summary Fetching
// ==========================================

pub fn _get_rating_summary(
    conn: &mut SqliteConnection,
    r_type: &str,
    t_id: &str,
) -> QueryResult<RatingSummary> {
    rating_summary::table
        .find((r_type.to_string(), t_id.to_string()))
        .first::<RatingSummary>(conn)
}

// ==========================================
// Recompute All Summaries (SQLite-safe)
// ==========================================

// Helper struct for in-memory aggregation
#[derive(Debug)]
struct Aggregated {
    rating_sum: i32,
    rating_count: i32,
    average_rating: f32,
    last_updated: String,
}

// Full rebuild: deletes nothing, just upserts
pub fn rebuild_rating_summary(conn: &mut SqliteConnection) -> QueryResult<()> {
    use crate::schema::rating_events::dsl::*;

    let events = rating_events
        .load::<RatingEvent>(conn)?;

    use std::collections::HashMap;
    let mut map: HashMap<(String, String), Aggregated> = HashMap::new();

    for evt in events {
        let key = (evt.rating_type.clone(), evt.target_id.clone());

        let entry = map.entry(key).or_insert(Aggregated {
            rating_sum: 0,
            rating_count: 0,
            average_rating: 0.0,
            last_updated: evt.created_at.clone(),
        });

        entry.rating_sum += evt.rating;
        entry.rating_count += 1;

        // Update latest timestamp
        if evt.created_at > entry.last_updated {
            entry.last_updated = evt.created_at.clone();
        }

        entry.average_rating =
            entry.rating_sum as f32 / entry.rating_count as f32;
    }

    // Now upsert into SQLite
    for ((r_type, t_id), agg) in map.into_iter() {
        upsert_summary_row(
            conn,
            &r_type,
            &t_id,
            agg.rating_sum,
            agg.rating_count,
            agg.average_rating,
            &agg.last_updated,
        )?;
    }

    Ok(())
}

// ==========================================
// SQLite-friendly UPSERT
// ==========================================

pub fn upsert_summary_row(
    conn: &mut SqliteConnection,
    r_type: &str,
    t_id: &str,
    r_sum: i32,
    r_count: i32,
    avg: f32,
    last: &str,
) -> QueryResult<()> {
    use crate::schema::rating_summary::dsl::*;

    // Try existing row
    let existing = rating_summary
        .filter(rating_type.eq(r_type))
        .filter(target_id.eq(t_id))
        .first::<RatingSummary>(conn)
        .optional()?;

    match existing {
        Some(_) => {
            diesel::update(
                rating_summary
                    .filter(rating_type.eq(r_type))
                    .filter(target_id.eq(t_id)),
            )
            .set((
                rating_sum.eq(r_sum),
                rating_count.eq(r_count),
                average_rating.eq(avg),
                last_updated.eq(last),
            ))
            .execute(conn)?;
        }

        None => {
            let new_row = NewRatingSummary {
                rating_type: r_type.to_string(),
                target_id: t_id.to_string(),
                rating_sum: r_sum,
                rating_count: r_count,
                average_rating: avg,
                last_updated: last.to_string(),
            };

            diesel::insert_into(rating_summary)
                .values(new_row)
                .execute(conn)?;
        }
    }

    Ok(())
}

// ==========================================
// Simple incremental updater (existing logic)
// ==========================================

pub fn _upsert_rating_summary(
    conn: &mut SqliteConnection,
    r_type: &str,
    t_id: &str,
    new_rating: i32,
) -> QueryResult<()> {
    use crate::schema::rating_summary::dsl::*;

    let existing = rating_summary
        .filter(rating_type.eq(r_type))
        .filter(target_id.eq(t_id))
        .first::<RatingSummary>(conn)
        .optional()?;

    let now = Utc::now().naive_utc().to_string();

    match existing {
        Some(mut summary) => {
            summary.rating_sum += new_rating;
            summary.rating_count += 1;
            summary.average_rating =
                summary.rating_sum as f32 / summary.rating_count as f32;
            summary.last_updated = now.clone();

            diesel::update(
                rating_summary
                    .filter(rating_type.eq(r_type))
                    .filter(target_id.eq(t_id)),
            )
            .set((
                rating_sum.eq(summary.rating_sum),
                rating_count.eq(summary.rating_count),
                average_rating.eq(summary.average_rating),
                last_updated.eq(summary.last_updated),
            ))
            .execute(conn)?;
        }
        None => {
            let new_summary = NewRatingSummary {
                rating_type: r_type.to_string(),
                target_id: t_id.to_string(),
                rating_sum: new_rating,
                rating_count: 1,
                average_rating: new_rating as f32,
                last_updated: now,
            };

            diesel::insert_into(rating_summary)
                .values(new_summary)
                .execute(conn)?;
        }
    }

    Ok(())
}



pub fn get_all_summaries(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<RatingSummary>> {
    use crate::schema::rating_summary::dsl::*;
    rating_summary.load::<RatingSummary>(conn)
}



#[derive(Serialize)]
pub struct SummaryItem {
    pub rating: f32,
    pub rating_count: i32,
    pub last_updated: String,
}


pub fn get_summary_map(
    conn: &mut SqliteConnection
) -> QueryResult<HashMap<String, SummaryItem>> {
    use crate::schema::rating_summary::dsl::*;

    let rows = rating_summary.load::<RatingSummary>(conn)?;

    let mut map = HashMap::new();

    for row in rows {
        map.insert(
            row.target_id.clone(),
            SummaryItem {
                rating: row.average_rating,
                rating_count: row.rating_count,
                last_updated: row.last_updated.clone(),
            }
        );
    }

    Ok(map)
}
