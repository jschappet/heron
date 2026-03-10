use crate::domains::weekly_reflection_domain::{QuestionAnswers, WeeklyReflectionDomain};
use crate::routes::register;
use crate::types::method::Method;
use actix_web::{HttpResponse, Responder, Scope, web};
use serde::Deserialize;

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

async fn submit_weekly_answers(
    domain: web::Data<WeeklyReflectionDomain>,
    payload: web::Json<WeeklyAnswersPayload>,
) -> impl Responder {
    let answers = payload
        .answers
        .iter()
        .map(|a| (a.uuid.clone(), a.answer.clone()))
        .collect();

    match domain.submit(payload.name.clone(), payload.email.clone(), answers) {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_answers(
    domain: web::Data<WeeklyReflectionDomain>,
    in_question_uuid: web::Path<String>,
) -> impl Responder {
    match domain.answers_for_question(&in_question_uuid) {
        Ok(ans) => HttpResponse::Ok().json(ans),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct AllAnswersQuery {
    min_answers: Option<usize>, // Optional parameter
}

async fn get_all_answers(
    domain: web::Data<WeeklyReflectionDomain>,
    query: web::Query<AllAnswersQuery>,
) -> impl Responder {
    let min_answers = query.min_answers.unwrap_or(3);

    match domain.get_all_answers(min_answers) {
        Ok(ans) => HttpResponse::Ok().json(ans),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_response(
    domain: web::Data<WeeklyReflectionDomain>,
    in_path: web::Path<(String, i32)>,
    
) -> impl Responder {
    let (in_uuid, in_reponse_id) = in_path.into_inner();

    match domain.get_response(&in_uuid, in_reponse_id) {
        Ok(ans) => HttpResponse::Ok().json(ans),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }

} 
async fn get_list_of_respondance(
    domain: web::Data<WeeklyReflectionDomain>,
    
) -> impl Responder {
    
    match domain.get_all_responders() {
        Ok(ans) => HttpResponse::Ok().json(ans),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize, Clone)]
struct UploadedQuestion {
    uuid: String,
    question: String,
}

// #[post("/questions/upload")]
async fn upload_questions(
    domain: web::Data<WeeklyReflectionDomain>,
    payload: web::Json<Vec<UploadedQuestion>>,
) -> impl Responder {
    let q = payload.into_inner();
    let questions_to_save: Vec<QuestionAnswers> = q
        .into_iter()
        .map(|q| QuestionAnswers {
            question_uuid: q.uuid,
            question: Some(q.question),
            answers: vec![], // empty initially
        })
        .collect();

    match domain.upload_questions(questions_to_save) {
        Ok(saved) => HttpResponse::Ok().json(saved),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        // POST / (submit weekly answers)
        .service(register(
            "weekly_answers_submit",
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

    // // POST /question/{uuid}/summary
    // .service(register(
    //     "submit_question_summary",
    //     Method::POST,
    //     &full_path,
    //     "question/{uuid}/summary",
    //     submit_summary,
    //     crate::types::MemberRole::Admin,
    // ))

    // // GET /question/{uuid}/summary
    // .service(register(
    //     "get_question_summary",
    //     Method::GET,
    //     &full_path,
    //     "question/{uuid}/summary",
    //     get_summary,
    //     crate::types::MemberRole::Public,
    // ))

    // // GET /question-summaries/all
    // .service(register(
    //     "get_all_question_summaries",
    //     Method::GET,
    //     &full_path,
    //     "question-summaries/all",
    //     get_all_summaries,
    //     crate::types::MemberRole::Public,
    // ))

    // // POST /question-summaries/update
    // .service(register(
    //     "update_question_summary",
    //     Method::POST,
    //     &full_path,
    //     "question-summaries/update",
    //     post_question_summary,
    //     crate::types::MemberRole::Admin,
    // ))
}

pub fn admin_scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        // GET /all (all answers)
        .service(register(
            "weekly_answers_all_respondance",
            Method::GET,
            &full_path,
            "all",
            get_list_of_respondance,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "weekly_answers_get_response",
            Method::GET,
            &full_path,
            "response/{uuid}/{id}",
            get_response,
            crate::types::MemberRole::Admin,
        ))
        .service(register(
            "get_all_weekly_answers",
            Method::GET,
            &full_path,
            "all",
            get_all_answers,
            crate::types::MemberRole::Admin,
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
}

//    .service(submit_weekly_answers)
//    .service(get_answers)
//    .service(submit_summary)
//    .service(get_summary)
//    .service(get_all_answers)
//    .service(upload_questions)
//    .service(post_question_summary)
//    .service(get_all_summaries)
