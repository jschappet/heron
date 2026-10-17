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
            .filter(drafts::status.ne("deleted"))
            .filter(drafts::submitted_by.eq(member_id))
            .select(DraftListItem::as_select())
            .load(conn)
    }

    // CREATE
    pub fn create_draft(conn: &mut SqliteConnection, new: &NewDraft) -> QueryResult<Draft> {
        // Keep insertion and retrieval under the same write lock. Another site's
        // concurrent insert must never become this request's response.
        conn.immediate_transaction(|conn| {
            diesel::insert_into(drafts::table)
                .values(new)
                .execute(conn)?;
            drafts::table.order(drafts::id.desc()).first(conn)
        })
    }
}

#[cfg(test)]
mod creation_tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    #[test]
    fn concurrent_sites_receive_their_own_created_draft() {
        let path = std::env::temp_dir().join(format!("heron-draft-{}.sqlite", uuid::Uuid::new_v4()));
        {
            let mut conn = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
            conn.batch_execute(include_str!("../../migrations/2026-01-04-170231_drafts/up.sql")).unwrap();
            conn.batch_execute(include_str!("../../migrations/2026-02-28-153103_addHostIdDraft/up.sql")).unwrap();
        }
        let workers: Vec<_> = (1..=2).map(|host_id| {
            let path = path.clone();
            std::thread::spawn(move || {
                let mut conn = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
                conn.batch_execute("PRAGMA busy_timeout = 5000;").unwrap();
                for index in 0..20 {
                    let title = format!("Site {host_id}, draft {index}");
                    let payload: NewDraft = serde_json::from_value(serde_json::json!({
                        "doc_type":"post", "title":title, "body_md":"", "submitted_by":host_id, "host_id":host_id
                    })).unwrap();
                    let draft = DraftService::create_draft(&mut conn, &payload).unwrap();
                    assert_eq!(draft.host_id, host_id);
                    assert_eq!(draft.submitted_by, host_id);
                    assert_eq!(draft.title, title);
                }
            })
        }).collect();
        for worker in workers { worker.join().unwrap(); }
        std::fs::remove_file(path).unwrap();
    }
}
