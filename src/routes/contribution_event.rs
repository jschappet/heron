use crate::domains::ledger_domain::LedgerDomain;
use crate::middleware::host::HostContext;
use crate::routes::register;
use crate::types::{Audience, MemberRole};
use crate::validator::{AuthContext, has_role};
use crate::errors::app_error::AppError;
//use crate::models::contribution::ContributionEventInput;
//use crate::schema::contribution_events::contributor_id;
use crate::services::contribute_events::{ContributionDomain, NewContributionEvent};
use actix_web::{HttpResponse,  Scope, web};
use serde::{Deserialize, Serialize};
use crate::types::method::Method;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionPayload {
    pub context_id: Option<String>,
    pub context_name: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub availability_days: Option<String>,
    pub availability_times: Option<String>,
    pub resource_type: Option<String>,
    pub quantity_unit: Option<String>,
    pub notes: Option<String>,
    pub effort: Option<String>,
    pub quantity_value: Option<f32>,

}


pub async fn create_contribute_event(
    contributions: web::Data<ContributionDomain>,
    payload: web::Json<ContributionPayload>,
    host: HostContext,
) -> Result<HttpResponse, AppError> {

        let payload = payload.into_inner();
        log::info!("Creating contributor with name: {} and Notes: {}", 
            payload.name.clone().unwrap_or_else(|| "Anonymous".into()), 
            payload.effort.clone().unwrap_or_else(|| "No notes provided".into()));

        let payload = NewContributionEvent {
            context_id: payload.context_id,
            context_name: payload.context_name,
            host_id: host.0.id, // This should be set to the authenticated user's host ID
            name: payload.name,
            email: payload.email,
            resource_type: payload.resource_type,
            quantity_unit: payload.quantity_unit,
            availability_days: payload.availability_days,
            availability_times: payload.availability_times,
            notes: payload.effort.clone(),
            effort: payload.effort,
            quantity_value: payload.quantity_value.unwrap_or(0.0),
        };


    let result = contributions
        .create_event(payload)?;

    Ok(HttpResponse::Ok().json(result))
}


pub async fn list_all_effort_context(
    //contributions: web::Data<ContributionDomain>,
    domain: web::Data<LedgerDomain>,
    admin: AuthContext,
    host: HostContext,
) -> Result<HttpResponse, AppError> {

    // Map the admin roles into an Audience
    let audience = if has_role(&admin.get_roles(),
         &[MemberRole::Admin]) {
        Audience::Admin
    } else {
        Audience::Authenticated
    };

    let result =
     domain.get_effort_contexts(host.0.id,audience)?;
    

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
        .service(register(
            "list_all_effort_context",
            Method::GET,
            &full_path,
            "/contexts",
            list_all_effort_context,
            crate::types::MemberRole::Public,
        ))
}


pub fn admin_scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        
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
