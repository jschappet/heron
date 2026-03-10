use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

use crate::errors::app_error::AppError;
use crate::models::weekly_answer::WeeklyAnswer;
use crate::schema::question_summaries;
use crate::schema::weekly_answers;
use std::collections::HashMap;

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = question_summaries)]
pub struct QuestionSummary {
    pub id: i32,
    pub question_uuid: String,
    pub answers_count: i32,
    pub question_text: String,
    pub summary: String,
    pub prompt: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = question_summaries)]
pub struct NewQuestionSummary {
    pub question_uuid: String,
    pub answers_count: i32,
    pub question_text: String,
    pub summary: String,
    pub prompt: String,
}

#[derive(Serialize)]
pub struct QuestionResponder {
    pub question_uuids: Vec<String>,
    pub response_ids: Vec<i32>,
    pub name: String,
    pub email: String,
}

impl QuestionSummary {
    pub fn by_question(conn: &mut SqliteConnection, uuid: &str) -> QueryResult<Self> {
        question_summaries::table
            .filter(question_summaries::question_uuid.eq(uuid))
            .first(conn)
    }

    pub fn get_response(conn: &mut SqliteConnection, uuid: &str, response_id: i32) -> 
        Result<(String, String), AppError> {

            let question = QuestionSummary::by_question(conn, uuid)?;
            let answer = WeeklyAnswer::get_answer(conn, response_id)?;
            Ok((question.question_text, answer))
        }

    pub fn responders_list(
        conn: &mut SqliteConnection,
    ) -> Result<Vec<QuestionResponder>, AppError> {
        let rows = weekly_answers::table
            .select((
                weekly_answers::id,
                weekly_answers::name,
                weekly_answers::email,
                weekly_answers::question_uuid,
            ))
            .order(weekly_answers::email.asc())
            .load::<(i32, String, String, String)>(conn)?;

        let mut map: HashMap<String, QuestionResponder> = HashMap::new();

        for (id, name, email, question_uuid) in rows {
            let entry = map
                .entry(email.clone())
                .or_insert_with(|| QuestionResponder {
                    question_uuids: Vec::new(),
                    response_ids: Vec::new(),
                    name: name.clone(),
                    email: email.clone(),
                });

            entry.question_uuids.push(question_uuid);
            entry.response_ids.push(id);
        }

        Ok(map.into_values().collect())
    }

    pub fn insert(conn: &mut SqliteConnection, new: &NewQuestionSummary) -> QueryResult<Self> {
        diesel::insert_into(question_summaries::table)
            .values(new)
            .execute(conn)?;

        Self::by_question(conn, &new.question_uuid)
    }

    pub fn update_summary(
        conn: &mut SqliteConnection,
        uuid: &str,
        new_summary: &str,
        new_prompt: &str,
        new_count: i32,
    ) -> QueryResult<Self> {
        use crate::schema::question_summaries::dsl::*;

        diesel::update(question_summaries.filter(question_uuid.eq(uuid)))
            .set((
                summary.eq(new_summary),
                prompt.eq(new_prompt),
                answers_count.eq(new_count),
            ))
            .execute(conn)?;

        Self::by_question(conn, uuid)
    }

    pub fn upsert_many(conn: &mut SqliteConnection, new_entries: Vec<NewQuestionSummary>) -> usize {
        use crate::schema::question_summaries::dsl::*;
        let mut inserted = 0;

        for entry in new_entries {
            let _ = diesel::insert_into(question_summaries)
                .values(&entry)
                .on_conflict(question_uuid)
                .do_nothing()
                .execute(conn)
                .map(|count| inserted += count)
                .unwrap_or_else(|e| eprintln!("Insert failed for {}: {}", entry.question_uuid, e));
        }

        inserted
    }
}
