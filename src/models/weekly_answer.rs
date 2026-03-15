use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{errors::app_error::AppError, schema::weekly_answers};

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = weekly_answers)]
pub struct WeeklyAnswer {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub question_uuid: String,
    pub answer: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = weekly_answers)]
pub struct NewWeeklyAnswer {
    pub name: String,
    pub email: String,
    pub question_uuid: String,
    pub answer: String,
}

impl WeeklyAnswer {
    pub fn insert_many(
        conn: &mut SqliteConnection,
        answers: &[NewWeeklyAnswer],
    ) -> QueryResult<usize> {
        diesel::insert_into(weekly_answers::table)
            .values(answers)
            .execute(conn)
    }

    pub fn by_question(
        conn: &mut SqliteConnection,
        uuid: &str,
    ) -> QueryResult<Vec<Self>> {
        weekly_answers::table
            .filter(weekly_answers::question_uuid.eq(uuid))
            .load(conn)
    }

    pub fn get_answer(conn: &mut SqliteConnection,
        response_id: i32) -> Result<String, AppError> {
                let answer = weekly_answers::table
                    .find(response_id).first::<WeeklyAnswer>(conn)?;
                Ok(answer.answer)

        }

    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        weekly_answers::table.load(conn)
    }
}