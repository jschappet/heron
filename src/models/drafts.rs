
use crate::{
    routes::drafts_api::{self, DraftQuery},
    schema::drafts,
    types::{DocType, DraftStatus, JsonField},
};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = drafts)]

pub struct Draft {
    pub id: i32,
    pub doc_type: DocType,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub author: Option<String>,
    pub meta: Option<JsonField>,
    pub body_md: String,
    pub status: String,
    pub submitted_by: i32,
    pub submitted_at: Option<chrono::NaiveDateTime>,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<chrono::NaiveDateTime>,
    pub review_notes: Option<String>,
    pub details: Option<String>,
}


impl Draft {
    pub fn get_field_value(&self, key: &str) -> Option<String> {
        match key {
            "title" => Some(self.title.clone()),
            "description" => self.description.clone(),
            "author" => self.author.clone(),
            _ => None,
        }
    }
}


#[derive(AsChangeset, Insertable, Deserialize, Serialize)]
#[diesel(table_name = drafts)]
pub struct NewDraft {
    pub doc_type: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub author: Option<String>,
    pub body_md: String,
    
    pub meta: Option<JsonField>,
    pub status: Option<String>,
    pub submitted_by: Option<i32>,
    pub submitted_at: Option<chrono::NaiveDateTime>,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<chrono::NaiveDateTime>,
    pub review_notes: Option<String>,
    pub details: Option<String>,
}

// CREATE
pub fn create_draft(conn: &mut SqliteConnection, new: &NewDraft) -> QueryResult<Draft> {
    diesel::insert_into(drafts::table)
        .values(new)
        .execute(conn)?;

    drafts::table.order(drafts::id.desc()).first(conn)
}

// READ all
pub fn _get_drafts(conn: &mut SqliteConnection) -> QueryResult<Vec<Draft>> {
    //drafts::table.load(conn)
    drafts::table.select(Draft::as_select()).load(conn)
}

pub fn _get_drafts_for_user(
    conn: &mut SqliteConnection,
    other_user_id: i32,
) -> QueryResult<Vec<Draft>> {
    drafts::table
        .filter(drafts::submitted_by.eq(other_user_id))
        .load(conn)
}

pub struct DraftFilter {
    pub status: Option<DraftStatus>,
    pub doc_type: Option<String>,
    pub author: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub submitted_by: Option<i32>,
}

impl From<DraftQuery> for DraftFilter {
    fn from(query: drafts_api::DraftQuery) -> Self {
        DraftFilter {
            status: query.status,
            doc_type: query.docType.clone(),
            author: query.author.clone(),
            date_from: query
                .dateFrom
                .as_deref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
            date_to: query
                .dateTo
                .as_deref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
            
            submitted_by: None,
        }
    }
}

pub fn get_drafts_filtered(
    conn: &mut SqliteConnection,
    filter: DraftFilter,
) -> QueryResult<Vec<Draft>> {
    let mut q = drafts::table.into_boxed();

    if let Some(status) = filter.status {
        log::info!("Setting Status: {:?}", status);
        q = q.filter(drafts::status.eq(status));
    }

    if let Some(uid) = filter.submitted_by {
        log::info!("Setting Submitted By: {}", uid);
        q = q.filter(drafts::submitted_by.eq(uid));
    }

       if let Some(author) = filter.author {
        log::info!("Setting Author: {}", author);
        q = q.filter(drafts::author.like(format!("{}%",author)));
    }

    if let Some(doc_type) = filter.doc_type {
        log::info!("Settings doc_type: '{}'", doc_type);

        if doc_type != "" {
            q = q.filter(drafts::doc_type.eq(doc_type));
        }
    }

    //use chrono::{NaiveDateTime};

    if let Some(date) = filter.date_from {
        let start = date.and_hms_opt(0, 0, 0).unwrap();
        q = q.filter(drafts::submitted_at.ge(start));
    }

    if let Some(date) = filter.date_to {
        
        let end = date.and_hms_opt(0, 0, 0).unwrap();
        log::info!("Date To: {}", date);
        q = q.filter(drafts::submitted_at.le(end));
    }

    q.load(conn)
}

#[allow(dead_code)]


pub fn _get_drafts_by_status(
    conn: &mut SqliteConnection,
    _filter: DraftFilter,
    other_user_id: Option<i32>,
) -> QueryResult<Vec<Draft>> {
    let mut query = drafts::table
        .filter(drafts::status.eq("approved"))
        .into_boxed();

    if let Some(uid) = other_user_id {
        query = query.filter(drafts::submitted_by.eq(uid));
    }

    query.load(conn)
}

// READ single
pub fn get_draft(conn: &mut SqliteConnection, in_draft_id: i32) -> QueryResult<Draft> {
    drafts::table.find(in_draft_id).first(conn)
}

// UPDATE
pub fn update_draft(
    conn: &mut SqliteConnection,
    in_draft_id: i32,
    updated: &NewDraft,
) -> QueryResult<Draft> {
    diesel::update(drafts::table.find(in_draft_id))
        .set(updated)
        .execute(conn)?;

    drafts::table.find(in_draft_id).first(conn)
}

// DELETE
pub fn delete_draft(conn: &mut SqliteConnection, in_draft_id: i32) -> QueryResult<usize> {
    diesel::delete(drafts::table.find(in_draft_id)).execute(conn)
}

use chrono::NaiveDate;


// Change status to submitted
pub fn submit_draft(conn: &mut SqliteConnection, draft_id: i32) -> QueryResult<Draft> {
    diesel::update(drafts::table.find(draft_id))
        .set((
            drafts::status.eq("submitted"),
            drafts::submitted_at.eq(chrono::Utc::now().naive_utc()),
        ))
        .execute(conn)?;

    drafts::table.find(draft_id).first(conn)
}

// Request changes from the author
pub fn request_changes(
    conn: &mut SqliteConnection,
    draft_id: i32,
    reviewer_id: i32,
    notes: &str,
) -> QueryResult<Draft> {
    diesel::update(drafts::table.find(draft_id))
        .set((
            drafts::status.eq("changes_requested"),
            drafts::reviewed_by.eq(Some(reviewer_id)),
            drafts::review_notes.eq(Some(notes.to_string())),
            drafts::reviewed_at.eq(Some(chrono::Utc::now().naive_utc())),
        ))
        .execute(conn)?;

    drafts::table.find(draft_id).first(conn)
}

// Approve a draft
pub fn approve_draft(
    conn: &mut SqliteConnection,
    draft_id: i32,
    reviewer_id: i32,
) -> QueryResult<Draft> {
    diesel::update(drafts::table.find(draft_id))
        .set((
            drafts::status.eq("approved"),
            drafts::reviewed_by.eq(Some(reviewer_id)),
            drafts::reviewed_at.eq(Some(chrono::Utc::now().naive_utc())),
        ))
        .execute(conn)?;

    drafts::table.find(draft_id).first(conn)
}

pub fn approve_drafts(
    conn: &mut SqliteConnection,
    ids: &[i32],
    in_reviewed_by: i32,
) -> QueryResult<usize> {
    use crate::schema::drafts::dsl::*;

    diesel::update(drafts.filter(id.eq_any(ids)))
        .set((
            status.eq("approved"),
            reviewed_by.eq(Some(in_reviewed_by)),
            reviewed_at.eq(Some(chrono::Utc::now().naive_utc())),
        ))
        .execute(conn)
}

// Deploy a draft
pub fn deploy_draft(
    conn: &mut SqliteConnection,
    draft_id: i32,
    reviewer_id: i32,
) -> QueryResult<Draft> {
    diesel::update(drafts::table.find(draft_id))
        .set((
            drafts::status.eq(DraftStatus::Deployed),
            drafts::reviewed_by.eq(Some(reviewer_id)),
            drafts::reviewed_at.eq(Some(chrono::Utc::now().naive_utc())),
        ))
        .execute(conn)?;

    drafts::table.find(draft_id).first(conn)
}
