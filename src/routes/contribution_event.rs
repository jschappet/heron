use crate::routes::register;
use crate::types::{Audience, MemberRole};
use crate::validator::{AuthContext, has_role};
use crate::{db::DbPool, models::contribution::ContributionEventInput};
use crate::errors::app_error::AppError;
//use crate::models::contribution::ContributionEventInput;
//use crate::schema::contribution_events::contributor_id;
use crate::services::contribute_events::{ContributionDomain, ContributionEventsService, NewContributionEvent};
use actix_web::{HttpResponse,  Scope, post, get, web};
use serde::{Deserialize, Serialize};
use crate::types::method::Method;


pub async fn create_contribute_event(
    contributions: web::Data<ContributionDomain>,
    payload: web::Json<NewContributionEvent>,
) -> Result<HttpResponse, AppError> {

    let result = contributions
        .create_event(payload.into_inner())?;

    Ok(HttpResponse::Ok().json(result))
}


pub async fn list_all_effort_context(
    contributions: web::Data<ContributionDomain>,
    admin: AuthContext
) -> Result<HttpResponse, AppError> {

    // Map the admin roles into an Audience
    let audience = if has_role(&admin.get_roles(), &[MemberRole::Admin]) {
        Audience::Admin
    } else {
        Audience::Authenticated
    };

    let result =
     contributions.get_effort_contexts(audience)?;
    

    Ok(HttpResponse::Ok().json(result))
}



pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        .service(register(
            "create_contribute_event",
            Method::POST,
            &full_path,
            "/efforts",
            create_contribute_event,
            crate::types::MemberRole::Public,
        ))
}


pub fn admin_scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        .service(register(
            "list_all_effort_context",
            Method::GET,
            &full_path,
            "",
            list_all_effort_context,
            crate::types::MemberRole::Admin,
        ))
}



/* 

#[post("/efforts_off")]
pub async fn _create_contribute_event(
    db_pool: web::Data<DbPool>,
    payload: web::Json<NewContributionEvent>,
) -> Result<HttpResponse, AppError> {
    let service = ContributionEventsService::new(db_pool.get_ref().clone());
    let payload = payload.into_inner();

    let context_id: i32 = match payload.context_id {
        Some(id) => service.get_id_from_short_code(&id)?,
        None => service.get_id_or_create_context(&payload.context_name.unwrap())?,
    };
    log::info!("Creating contributor with name: {} and email: {}", payload.name.clone().unwrap_or_else(|| "Anonymous".into()), 
        payload.email.clone().unwrap_or_else(|| "blank@nobody.com".into()));

    let contrib_id = service.get_id_or_create_contributor(
        &payload.name.unwrap_or_else(|| "Anonymous".into()),
        &payload.email.unwrap_or_else(|| "blank@nobody.com".into()),
    )?;

    let event = ContributionEventInput {
        context_id: context_id,
        contributor_id: contrib_id, // This should be set to the authenticated user's ID
        effort_date: Some(chrono::Utc::now().naive_utc()),
        hours: None,
        work_done: payload.effort.clone().unwrap_or_default(),
        details: payload.notes,
        appreciation_message: Some(String::new()), // This can be set based on your logic
        public_flag: Some(false),                  // Set this based on your requirements
    };
    // Call the service layer
    match service.create_event(&event) {
        Ok(saved_event) => Ok(HttpResponse::Ok().json(saved_event)),
        Err(err) => Err(err),
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewContributionEvent {
    pub context_id: Option<String>,
    pub context_name: Option<String>, // the offer the user is contributing to
    pub name: Option<String>,         // user name
    pub email: Option<String>,        // user email
    //pub how_helping: Option<String>,  // description of how they can help
    pub availability_days: Option<String>, // days they are available
    pub availability_times: Option<String>, // times they are available
    pub notes: Option<String>,        // any extra notes
    pub effort: Option<String>, // when the effort took place
}
*/
