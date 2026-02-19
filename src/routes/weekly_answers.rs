use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use actix_web::{HttpResponse, Responder, Scope, get, post, web};
//use crate::schema::question_summaries::question_text;
use crate::schema::{weekly_answers, question_summaries};
use crate::routes::register;
use crate::types::method::Method;

use crate::app_state::AppState;
//use diesel::upsert::excluded;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::weekly_answers)]
pub struct WeeklyAnswer {
    pub id: Option<i32>,
    pub name: String,
    pub email: String,
    pub question_uuid: String,
    pub answer: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = weekly_answers)]
pub struct NewWeeklyAnswer {
pub name: String,
pub email: String,
pub question_uuid: String,
pub answer: String,
}


#[derive(Debug, Deserialize)]
pub struct WeeklyAnswersPayload {
    pub name: String,
    pub email: String,
    pub answers: Vec<NewWeeklyAnswerItem>,
}

#[derive(Debug, Deserialize)]
pub struct NewWeeklyAnswerItem {
    pub uuid: String,
    pub answer: String,
}

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::question_summaries)]
pub struct QuestionSummary {
    pub id: Option<i32>,
    pub question_uuid: String,
    pub answers_count: i32,
    pub question_text: String,
    pub summary: String,
    pub prompt: String,
    pub created_at: Option<NaiveDateTime>,
}



#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = question_summaries)]
pub struct NewQuestionSummary {
    pub question_uuid: String,
    pub answers_count: i32,
    pub question_text: String,
    pub summary: String,
    pub prompt: String,
}

// CRUD functions
pub fn insert_weekly_answers(
    conn: &mut SqliteConnection,
    new_answers: Vec<NewWeeklyAnswer>
) -> QueryResult<usize> {
    diesel::insert_into(weekly_answers::table)
        .values(&new_answers)
        .execute(conn)
}

pub fn get_answers_by_question(
    conn: &mut SqliteConnection,
    question_uuid_val: &str
) -> QueryResult<Vec<WeeklyAnswer>> {
    weekly_answers::table
        .filter(weekly_answers::question_uuid.eq(question_uuid_val))
        .load::<WeeklyAnswer>(conn)
}

pub fn insert_question_summary(
    conn: &mut SqliteConnection,
    input_summary: NewQuestionSummary
) -> QueryResult<QuestionSummary> {
    diesel::insert_into(question_summaries::table)
        .values(&input_summary)
        .execute(conn)?;

    question_summaries::table
        .filter(question_summaries::question_uuid.eq(&input_summary.question_uuid))
        .first::<QuestionSummary>(conn)
}


//use diesel::prelude::*;

pub fn update_question_summary(
    conn: &mut SqliteConnection,
    question_uuid_val: &str,
    new_summary: &str,
    new_prompt: &str,
    new_answers_count: i32,
) -> QueryResult<QuestionSummary> {

    use crate::schema::question_summaries::dsl::{
        question_summaries,
        question_uuid,
        summary,
        answers_count,
        prompt
    };

    diesel::update(question_summaries.filter(question_uuid.eq(question_uuid_val)))
        .set((
            summary.eq(new_summary),
            answers_count.eq(new_answers_count),
            prompt.eq(new_prompt),
        ))
        .execute(conn)?;

    // Return the updated row
    question_summaries
        .filter(question_uuid.eq(question_uuid_val))
        .first::<QuestionSummary>(conn)
}


pub fn get_summary_by_question(
    conn: &mut SqliteConnection,
    question_uuid_val: &str
) -> QueryResult<QuestionSummary> {
    question_summaries::table
        .filter(question_summaries::question_uuid.eq(question_uuid_val))
        .first::<QuestionSummary>(conn)
}


pub fn get_all_weekly_answers(
    conn: &mut SqliteConnection,
) -> QueryResult<Vec<WeeklyAnswer>> {
    weekly_answers::table
        .load::<WeeklyAnswer>(conn)
}



// #[post("")]
async fn submit_weekly_answers(
    data: web::Data<AppState>,
    payload: web::Json<WeeklyAnswersPayload>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("DB connection failed");

    // Map answers into NewWeeklyAnswer including name/email
    let new_answers: Vec<NewWeeklyAnswer> = payload
        .answers
        .iter()
        .map(|a| NewWeeklyAnswer {
            name: payload.name.clone(),
            email: payload.email.clone(),
            question_uuid: a.uuid.clone(),
            answer: a.answer.clone(),
        })
        .collect();

    // Insert answers
    if let Err(err) = insert_weekly_answers(conn, new_answers) {
        return HttpResponse::InternalServerError().body(format!("DB error: {:?}", err));
    }

    // Build a response per question_uuid
    let mut summaries_response = Vec::new();

    for a in &payload.answers {
        match get_answers_by_question(conn, &a.uuid) {
            Ok(answers) => {
                if answers.len() < 3 {
                    summaries_response.push(serde_json::json!({
                        "question_uuid": a.uuid,
                        "summary": "We don't have enough responses to this question yet",
                        "answers_count": answers.len()
                    }));
                } else {
                    // Try to fetch AI-generated summary if exists
                    match get_summary_by_question(conn, &a.uuid) {
                        Ok(summary) => summaries_response.push(serde_json::json!({
                            "question_uuid": summary.question_uuid,
                            "summary": summary.summary,
                            "answers_count": summary.answers_count
                        })),
                        Err(_) => summaries_response.push(serde_json::json!({
                            "question_uuid": a.uuid,
                            "summary": "No AI summary available yet",
                            "answers_count": answers.len()
                        })),
                    }
                }
            }
            Err(_) => summaries_response.push(serde_json::json!({
                "question_uuid": a.uuid,
                "summary": "Error fetching answers",
                "answers_count": 0
            })),
        }
    }

    HttpResponse::Ok().json(summaries_response)
}

// #[get("/question/{uuid}/answers")]
async fn get_answers(
    data: web::Data<AppState>,
    in_question_uuid: web::Path<String>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("DB connection failed");
    match get_answers_by_question(conn, &in_question_uuid) {
        Ok(answers) => HttpResponse::Ok().json(answers),
        Err(err) => HttpResponse::InternalServerError().body(format!("DB error: {:?}", err)),
    }
}

// #[post("/question/{uuid}/summary")]
async fn submit_summary(
    data: web::Data<AppState>,
    in_question_uuid: web::Path<String>,
    new_summary: web::Json<NewQuestionSummary>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("DB connection failed");
    log::info!("Submitting summary for question UUID: {}", in_question_uuid);
    match insert_question_summary(conn, new_summary.into_inner()) {
        Ok(in_summary) => HttpResponse::Ok().json(in_summary),
        Err(err) => HttpResponse::InternalServerError().body(format!("DB error: {:?}", err)),
    }
}

// #[get("/question/{uuid}/summary")]
async fn get_summary(
    data: web::Data<AppState>,
    in_question_uuid: web::Path<String>,
) -> impl Responder {
    log::info!("Getting Question Summary: {}", in_question_uuid);
    let conn = &mut data.db_pool.get().expect("DB connection failed");
    match get_summary_by_question(conn, &in_question_uuid) {
        Ok(in_summary) => HttpResponse::Ok().json(in_summary),
        Err(err) => HttpResponse::InternalServerError().body(format!("DB error: {:?}", err)),
    }
}



#[derive(Serialize)]
struct QuestionAnswers {
    question_uuid: String,
    question: Option<String>,
    answers: Vec<String>,
}


#[derive(Deserialize)]
pub struct AllAnswersQuery {
    min_answers: Option<usize>, // Optional parameter
}

// #[get("/question-summaries/all")]
async fn get_all_summaries(
    data: web::Data<AppState>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("DB connection failed");  
    match question_summaries::table.load::<QuestionSummary>(conn) {
        Ok(summaries) => HttpResponse::Ok().json(summaries),
        Err(err) => HttpResponse::InternalServerError().body(format!("DB error: {:?}", err)),
    }   
}


// #[get("/all")]
async fn get_all_answers(
    data: web::Data<AppState>,
    query: web::Query<AllAnswersQuery>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("DB connection failed");
    let min_answers = query.min_answers.unwrap_or(3); // default to 3 if not provided

    // Step 1: Load all answers
    let all_answers: Vec<WeeklyAnswer> = match get_all_weekly_answers(conn) {
        Ok(list) => list,
        Err(e) => {
            eprintln!("DB Error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Step 2: Group by question_uuid
    use std::collections::HashMap;
    let mut grouped: HashMap<String, Vec<WeeklyAnswer>> = HashMap::new();
    for ans in all_answers {
        grouped
            .entry(ans.question_uuid.clone())
            .or_default()
            .push(ans);
    }

    // Step 3: Convert groups into QuestionAnswers and filter
    let mut output: Vec<QuestionAnswers> = Vec::new();
    for (uuid, group) in grouped {
        log::info!("Processing question UUID: {} with {} answers", uuid, group.len());
        if group.len() < min_answers {
            continue; // skip questions with fewer than 3 responses
        }


        let q_text = match get_summary_by_question(conn, uuid.as_str()) {
            Ok(in_summary) => Some(in_summary.question_text),
            Err(_) => None,
        };
            
        let answer_texts = group.into_iter().map(|a| a.answer).collect();

        output.push(QuestionAnswers {
            question_uuid: uuid,
            question: q_text,
            answers: answer_texts,
        });
    }

    HttpResponse::Ok().json(output)
}




#[derive(Deserialize)]
struct UploadedQuestion {
    uuid: String,
    question: String,
}


// #[post("/questions/upload")]
async fn upload_questions(
    data: web::Data<AppState>,
    payload: web::Json<Vec<UploadedQuestion>>,
) -> impl Responder {
    let conn = &mut data.db_pool.get().expect("DB connection failed");

    // Convert the uploaded questions into insertable structs
    let new_entries: Vec<NewQuestionSummary> = payload
        .iter()
        .map(|q| NewQuestionSummary {
            question_uuid: q.uuid.clone(),
            question_text: q.question.clone(),
            answers_count: 0,
            summary: "".to_string(),
            prompt: "".to_string(),
        })
        .collect();


   let mut inserted = 0;

for entry in new_entries {
    let res = diesel::insert_into(question_summaries::table)
        .values(&entry)
        .on_conflict(question_summaries::question_uuid)
        .do_update()
        .set(question_summaries::question_text.eq(entry.question_text.clone()))
        .execute(conn);

    if let Ok(count) = res {
        inserted += count;
    } else {
        eprintln!("Insert/update failed for {:?}", entry.question_uuid);
    }
}

HttpResponse::Ok().json(format!("Inserted or updated {} questions", inserted))

 /*    match result {
        Ok(count) => HttpResponse::Ok().json(format!("Inserted or updated {} questions", count)),
        Err(e) => {
            eprintln!("DB Error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to insert questions")
        }
    } */
}

#[derive(Deserialize)]
pub struct UpdateSummaryRequest {
    pub question_uuid: String,
    pub summary: String,
    pub answers_count: i32,
    pub prompt: String,
}

// #[post("/question-summaries/update")]
async fn post_question_summary(
    data: web::Data<AppState>,
    payload: web::Json<UpdateSummaryRequest>,
) -> impl Responder {
    let payload = payload.into_inner();
    let conn = &mut data.db_pool.get().expect("DB connection failed");

    let result = update_question_summary(conn,
        payload.question_uuid.as_str(),
        payload.summary.as_str(),
        payload.prompt.as_str(),
        payload.answers_count);


    match result {
        Ok(updated_summary) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "ok",
                "message": "Summary updated successfully",
                "summary": {
                    "id": updated_summary.id,
                    "question_uuid": updated_summary.question_uuid,
                    "answers_count": updated_summary.answers_count,
                    "summary": updated_summary.summary,
                }
            }))
        }

        Err(e) => {
            eprintln!("Database update error: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Failed to update summary",
                "details": e.to_string()
            }))
        }
    }

}

pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");    
    web::scope("")
    // POST / (submit weekly answers)
.service(register(
    "submit_weekly_answers",
    Method::POST,
    &full_path,
    "",
    submit_weekly_answers,
    crate::types::MemberRole::Public,
))

// GET /question/{uuid}/answers
.service(register(
    "get_answers_by_question",
    Method::GET,
    &full_path,
    "question/{uuid}/answers",
    get_answers,
    crate::types::MemberRole::Public,
))

// POST /question/{uuid}/summary
.service(register(
    "submit_question_summary",
    Method::POST,
    &full_path,
    "question/{uuid}/summary",
    submit_summary,
    crate::types::MemberRole::Admin,
))

// GET /question/{uuid}/summary
.service(register(
    "get_question_summary",
    Method::GET,
    &full_path,
    "question/{uuid}/summary",
    get_summary,
    crate::types::MemberRole::Public,
))

// GET /question-summaries/all
.service(register(
    "get_all_question_summaries",
    Method::GET,
    &full_path,
    "question-summaries/all",
    get_all_summaries,
    crate::types::MemberRole::Public,
))

// GET /all (all answers)
.service(register(
    "get_all_weekly_answers",
    Method::GET,
    &full_path,
    "all",
    get_all_answers,
    crate::types::MemberRole::Public,
))

// POST /questions/upload
.service(register(
    "upload_questions",
    Method::POST,
    &full_path,
    "questions/upload",
    upload_questions,
    crate::types::MemberRole::Admin,
))

// POST /question-summaries/update
.service(register(
    "update_question_summary",
    Method::POST,
    &full_path,
    "question-summaries/update",
    post_question_summary,
    crate::types::MemberRole::Admin,
))

 
}
    //    .service(submit_weekly_answers)
    //    .service(get_answers)
    //    .service(submit_summary)
    //    .service(get_summary)
    //    .service(get_all_answers)
    //    .service(upload_questions)
    //    .service(post_question_summary)
    //    .service(get_all_summaries)