

use std::collections::HashMap;

use itertools::Itertools;
use serde::Serialize;
use serde_json::{Value, json};

use crate::db::DbPool;
use crate::errors::app_error::AppError;
use crate::models::question_summary::{NewQuestionSummary, QuestionResponder, QuestionSummary};
use crate::models::weekly_answer::WeeklyAnswer;
use crate::services::dto::ReflectionSummaryDTO;
use crate::services::weekly_reflection_service::WeeklyReflectionService;

#[derive(Serialize)]
pub struct QuestionAnswers {
    pub question_uuid: String,
    pub question: Option<String>,
    pub answers: Vec<String>,
}



#[derive(Clone)]
pub struct WeeklyReflectionDomain {
    pool: DbPool,
}

impl WeeklyReflectionDomain {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<crate::db::DbConn, AppError> {
        self.pool.get().map_err(|e| AppError::User(e.to_string()))
    }

    pub fn submit(
        &self,
        name: String,
        email: String,
        answers: Vec<(String, String)>,
    ) -> Result<Vec<ReflectionSummaryDTO>, AppError> {
        let mut conn = self.conn()?;
        WeeklyReflectionService::submit_answers(&mut conn, name, email, answers)
    }

    pub fn answers_for_question(&self, uuid: &str) -> Result<Vec<WeeklyAnswer>, AppError> {
        let mut conn = self.conn()?;
        WeeklyReflectionService::get_answers(&mut conn, uuid)
    }

    pub fn get_all_answers(&self, min_answers: usize) -> Result<Vec<QuestionAnswers>, AppError> {
        let mut conn = self.conn()?;
        let all_answers = WeeklyReflectionService::get_all_answers(&mut conn)?;

        let grouped = all_answers
            .into_iter()
            .into_group_map_by(|a| a.question_uuid.clone());

        let output: Vec<QuestionAnswers> = grouped
            .into_iter()
            .filter_map(|(uuid, group)| {
                if group.len() < min_answers {
                    return None;
                }
                let question_text =
                    WeeklyReflectionService::get_summary_by_question(&mut conn, uuid.as_str());
                let answers = group.into_iter().map(|a| a.answer).collect();

                Some(QuestionAnswers {
                    question_uuid: uuid,
                    question: question_text,
                    answers,
                })
            })
            .collect();

        Ok(output)
    }

    pub fn upload_questions(&self, payload: Vec<QuestionAnswers>) -> Result<Value, AppError> {
        // Convert the uploaded questions into insertable structs
        let new_entries: Vec<NewQuestionSummary> = payload
            .iter()
            .map(|q| NewQuestionSummary {
                question_uuid: q.question_uuid.clone(),
                question_text: q.question.clone().unwrap_or("Not Listed".to_string()),
                answers_count: 0,
                summary: "".to_string(),
                prompt: "".to_string(),
            })
            .collect();

        let mut conn = self.conn()?;
        let inserted = QuestionSummary::upsert_many(&mut conn, new_entries);
        Ok(json!({"QuestionsAdded": inserted}))
    }

    pub fn get_all_responders(&self) -> Result<HashMap<String, QuestionResponder>, AppError> {
        let mut conn = self.conn()?;
        
        let responses = QuestionSummary::responders_list(&mut conn)?;
        let map: HashMap<String, QuestionResponder> =
            responses
                .into_iter()
                .map(|r| (r.name.clone(), r))
                .collect();
        Ok(map)
    }

    pub fn get_response(&self, question_uuid: &str, response_id: i32) -> Result<Value, AppError> {
                let mut conn = self.conn()?;

        let (question, response) = 
            QuestionSummary::get_response(&mut conn, question_uuid, response_id)?;
        Ok(json!({"question": question, "response": response}))
    }
}
