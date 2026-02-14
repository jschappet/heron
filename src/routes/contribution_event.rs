use crate::types::{Audience, MemberRole};
use crate::validator::{AdminContext, has_role};
use crate::{db::DbPool, models::contribution::ContributionEventInput};
use crate::errors::app_error::AppError;
//use crate::models::contribution::ContributionEventInput;
//use crate::schema::contribution_events::contributor_id;
use crate::services::contribute_events::{ContributionDomain, ContributionEventsService, NewContributionEvent};
use actix_web::{HttpResponse,  Scope, post, get, web};
use serde::{Deserialize, Serialize};


#[post("/efforts")]
pub async fn create_contribute_event(
    contributions: web::Data<ContributionDomain>,
    payload: web::Json<NewContributionEvent>,
) -> Result<HttpResponse, AppError> {

    let result = contributions
        .create_event(payload.into_inner())?;

    Ok(HttpResponse::Ok().json(result))
}

#[get("")]
pub async fn list_all_effort_context(
    contributions: web::Data<ContributionDomain>,
    admin: AdminContext
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



pub fn scope() -> Scope {
    web::scope("")
        .service(create_contribute_event)
}


pub fn admin_scope() -> Scope {
    web::scope("")
        .service(list_all_effort_context)
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
