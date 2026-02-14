use crate::{schema::{memberships::user_id, recipe_drafts}, types::DraftStatus};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct RecipeDraft {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub tags: String,
    pub author: String,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub total_time: Option<i32>,
    pub servings: Option<i32>,
    pub difficulty: Option<String>,
    pub source: Option<String>,
    pub dietary: Option<String>, // JSON string
    pub body_md: String,
    pub status: String,
    pub submitted_by: i32,
    pub submitted_at: Option<chrono::NaiveDateTime>,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<chrono::NaiveDateTime>,
    pub review_notes: Option<String>,
    pub details: Option<String>,
}

#[derive(AsChangeset, Insertable, Deserialize)]
#[diesel(table_name = recipe_drafts)]
pub struct NewRecipeDraft {
    pub title: String,
    pub description: String,
    pub tags: String,
    pub author: String,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub total_time: Option<i32>,
    pub servings: Option<i32>,
    pub difficulty: Option<String>,
    pub source: Option<String>,
    pub dietary: Option<String>,
    pub body_md: String,
    pub status: String,
    pub submitted_by: i32,
    pub submitted_at: Option<chrono::NaiveDateTime>,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<chrono::NaiveDateTime>,
    pub review_notes: Option<String>,
    pub details: Option<String>,
}

// CREATE
pub fn create_recipe_draft(
    conn: &mut SqliteConnection,
    new: &NewRecipeDraft,
) -> QueryResult<RecipeDraft> {
    diesel::insert_into(recipe_drafts::table)
        .values(new)
        .execute(conn)?;

    recipe_drafts::table
        .order(recipe_drafts::id.desc())
        .first(conn)
}

// READ all
pub fn get_recipe_drafts(conn: &mut SqliteConnection) -> QueryResult<Vec<RecipeDraft>> {
    recipe_drafts::table.load(conn)
}

pub fn get_recipe_drafts_for_user (
    conn: &mut SqliteConnection,
    other_user_id: i32,
) -> QueryResult<Vec<RecipeDraft>> {
    recipe_drafts::table
        .filter(recipe_drafts::submitted_by.eq(other_user_id))
        .load(conn)
}


pub fn get_recipe_drafts_by_status(
    conn: &mut SqliteConnection,
    status: &str,
    other_user_id: Option<i32>,
) -> QueryResult<Vec<RecipeDraft>> {
    let mut query = recipe_drafts::table
        .filter(recipe_drafts::status.eq(status))
        .into_boxed();

    if let Some(uid) = other_user_id {
        query = query.filter(recipe_drafts::submitted_by.eq(uid));
    }

    query.load(conn)
}

    

// READ single
pub fn get_recipe_draft(conn: &mut SqliteConnection, in_draft_id: i32) -> QueryResult<RecipeDraft> {
    recipe_drafts::table.find(in_draft_id).first(conn)
}

// UPDATE
pub fn update_recipe_draft(
    conn: &mut SqliteConnection,
    in_draft_id: i32,
    updated: &NewRecipeDraft,
) -> QueryResult<RecipeDraft> {
    diesel::update(recipe_drafts::table.find(in_draft_id))
        .set(updated)
        .execute(conn)?;

    recipe_drafts::table.find(in_draft_id).first(conn)
}

// DELETE
pub fn delete_recipe_draft(conn: &mut SqliteConnection, in_draft_id: i32) -> QueryResult<usize> {
    diesel::delete(recipe_drafts::table.find(in_draft_id)).execute(conn)
}

use chrono::NaiveDateTime;
use diesel::prelude::*;

// Change status to submitted
pub fn submit_draft(conn: &mut SqliteConnection, draft_id: i32) -> QueryResult<RecipeDraft> {
    diesel::update(recipe_drafts::table.find(draft_id))
        .set((
            recipe_drafts::status.eq("submitted"),
            recipe_drafts::submitted_at.eq(chrono::Utc::now().naive_utc()),
        ))
        .execute(conn)?;

    recipe_drafts::table.find(draft_id).first(conn)
}

// Request changes from the author
pub fn request_changes(
    conn: &mut SqliteConnection,
    draft_id: i32,
    reviewer_id: i32,
    notes: &str,
) -> QueryResult<RecipeDraft> {
    diesel::update(recipe_drafts::table.find(draft_id))
        .set((
            recipe_drafts::status.eq("changes_requested"),
            recipe_drafts::reviewed_by.eq(Some(reviewer_id)),
            recipe_drafts::review_notes.eq(Some(notes.to_string())),
            recipe_drafts::reviewed_at.eq(Some(chrono::Utc::now().naive_utc())),
        ))
        .execute(conn)?;

    recipe_drafts::table.find(draft_id).first(conn)
}

// Approve a draft
pub fn approve_draft(
    conn: &mut SqliteConnection,
    draft_id: i32,
    reviewer_id: i32,
) -> QueryResult<RecipeDraft> {
    diesel::update(recipe_drafts::table.find(draft_id))
        .set((
            recipe_drafts::status.eq("approved"),
            recipe_drafts::reviewed_by.eq(Some(reviewer_id)),
            recipe_drafts::reviewed_at.eq(Some(chrono::Utc::now().naive_utc())),
        ))
        .execute(conn)?;

    recipe_drafts::table.find(draft_id).first(conn)
}


pub fn approve_drafts(
    conn: &mut SqliteConnection,
    ids: &[i32],
    in_reviewed_by: i32,
) -> QueryResult<usize> {
    use crate::schema::recipe_drafts::dsl::*;

    diesel::update(recipe_drafts.filter(id.eq_any(ids)))
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
) -> QueryResult<RecipeDraft> {
    diesel::update(recipe_drafts::table.find(draft_id))
        .set((
            recipe_drafts::status.eq(DraftStatus::Deployed),
            recipe_drafts::reviewed_by.eq(Some(reviewer_id)),
            recipe_drafts::reviewed_at.eq(Some(chrono::Utc::now().naive_utc())),
        ))
        .execute(conn)?;

    recipe_drafts::table.find(draft_id).first(conn)
}
