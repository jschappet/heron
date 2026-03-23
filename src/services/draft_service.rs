use diesel::{QueryResult, Queryable, Selectable, SqliteConnection};
use serde::{Deserialize, Serialize};

use crate::{
    models::drafts::{Draft, NewDraft}, schema::drafts, types::{DocType, DraftStatus}
};
use diesel::prelude::*;

#[derive(Serialize, Deserialize, Debug, Selectable, Queryable)]
#[diesel(table_name = drafts)]

pub struct DraftListItem {
    pub id: i32,
    pub doc_type: DocType,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub status: DraftStatus,
    pub submitted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Clone)]
pub struct DraftService;

impl DraftService {
    pub fn get_draft_list_for_user(
        conn: &mut SqliteConnection,
        host_id: i32,
        member_id: i32,
    ) -> QueryResult<Vec<DraftListItem>> {
        drafts::table
            .filter(drafts::host_id.eq(host_id))
            .filter(drafts::submitted_by.eq(member_id))
            .select(DraftListItem::as_select())
            .load(conn)
    }

    // CREATE
    pub fn create_draft(conn: &mut SqliteConnection, new: &NewDraft) -> QueryResult<Draft> {
        diesel::insert_into(drafts::table)
            .values(new)
            .execute(conn)?;

        drafts::table.order(drafts::id.desc()).first(conn)
    }
}
