use diesel::sqlite::SqliteConnection;

use crate::errors::app_error::AppError;
use crate::models::weekly_answer::{NewWeeklyAnswer, WeeklyAnswer};
use crate::models::question_summary::{QuestionResponder, QuestionSummary};
use crate::services::dto::ReflectionSummaryDTO;

pub struct WeeklyReflectionService;

impl WeeklyReflectionService {
    pub fn submit_answers(
        conn: &mut SqliteConnection,
        name: String,
        email: String,
        answers: Vec<(String, String)>,
    ) -> Result<Vec<ReflectionSummaryDTO>, AppError> {
        let new_answers: Vec<NewWeeklyAnswer> = answers
            .iter()
            .map(|(uuid, answer)| NewWeeklyAnswer {
                name: name.clone(),
                email: email.clone(),
                question_uuid: uuid.clone(),
                answer: answer.clone(),
            })
            .collect();

        WeeklyAnswer::insert_many(conn, &new_answers)
            .map_err(AppError::Db)?;

        let mut response = Vec::new();

        for (uuid, _) in answers {
            let answers = WeeklyAnswer::by_question(conn, &uuid)
                .map_err(AppError::Db)?;

            if answers.len() < 3 {
                response.push(ReflectionSummaryDTO::insufficient(uuid, answers.len()));
                continue;
            }

            match QuestionSummary::by_question(conn, &uuid) {
                Ok(summary) => response.push(ReflectionSummaryDTO::from_summary(summary)),
                Err(_) => response.push(ReflectionSummaryDTO::pending(uuid, answers.len())),
            }
        }

        Ok(response)
    }

    pub fn get_answers(
        conn: &mut SqliteConnection,
        uuid: &str,
    ) -> Result<Vec<WeeklyAnswer>, AppError> {
        WeeklyAnswer::by_question(conn, uuid).map_err(AppError::Db)
    }

    pub fn get_all_answers(
        conn: &mut SqliteConnection,
    ) -> Result<Vec<WeeklyAnswer>, AppError> {
        WeeklyAnswer::all(conn).map_err(AppError::Db)
    }


    pub fn get_list_of_respondance(
        conn: &mut SqliteConnection,
        
    ) -> Result<Vec<QuestionResponder>, AppError> {
        QuestionSummary::responders_list(conn)
    }


        pub fn get_summary_by_question(
        conn: &mut SqliteConnection,
        question_uuid_val: &str
    ) -> Option<String> {
        
         match QuestionSummary::by_question(conn, question_uuid_val) {
                Ok(in_summary) => Some(in_summary.question_text),
                Err(e) => None,
            }
    }


}