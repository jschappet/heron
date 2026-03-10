use serde::Serialize;
use crate::models::question_summary::QuestionSummary;

#[derive(Serialize)]
pub struct ReflectionSummaryDTO {
    pub question_uuid: String,
    pub summary: String,
    pub answers_count: usize,
}

impl ReflectionSummaryDTO {
    pub fn insufficient(uuid: String, count: usize) -> Self {
        Self {
            question_uuid: uuid,
            summary: "We don't have enough responses to this question yet".into(),
            answers_count: count,
        }
    }

    pub fn pending(uuid: String, count: usize) -> Self {
        Self {
            question_uuid: uuid,
            summary: "No AI summary available yet".into(),
            answers_count: count,
        }
    }

    pub fn from_summary(summary: QuestionSummary) -> Self {
        Self {
            question_uuid: summary.question_uuid,
            summary: summary.summary,
            answers_count: summary.answers_count as usize,
        }
    }
}