use crate::errors::auth_error::AuthError;
use crate::models::recipe_drafts::*;
use crate::types::{DraftStatus, MemberRole};
use crate::validator::{AuthContext, has_role, require_role};
use crate::{AppState, schema::recipe_drafts::author};
use actix_web::{HttpResponse, Responder, delete, get, post, web};
use serde::Deserialize;

// Create or save draft
#[post("/recipe_drafts")]
pub async fn create_recipe_draft_api(
    data: web::Data<AppState>,
    new: web::Json<NewRecipeDraft>,
    auth_context: AuthContext,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    let mut new = new.into_inner();
    new.submitted_by = auth_context.user_id;

    match create_recipe_draft(&mut conn, &new) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        //.finish(),
    }
}

#[derive(Deserialize)]
pub struct DraftQuery {
    pub status: Option<DraftStatus>,
}

#[get("/recipe_drafts")]
pub async fn get_recipe_drafts_api(
    data: web::Data<AppState>,
    auth_context: AuthContext,
    query: web::Query<DraftQuery>,
) -> Result<HttpResponse, AuthError> {

    let mut conn = data.db_pool.get().unwrap();

    require_role(&auth_context.roles, &[MemberRole::Admin, MemberRole::Reviewer])?;
    
    let is_privileged = has_role(&auth_context.roles, &[MemberRole::Admin, MemberRole::Reviewer]);

    let user_filter = if is_privileged { None } else { Some(auth_context.user_id) };

    let result = match query.status {
        Some(status) => get_recipe_drafts_by_status(&mut conn, status.value(), user_filter),
        None => {
            if is_privileged {
                get_recipe_drafts(&mut conn)
            } else {
                get_recipe_drafts_for_user(&mut conn, auth_context.user_id)
            }
        }
    };

    match result {
        Ok(list) => Ok(HttpResponse::Ok().json(list)),
        Err(_) => Err(AuthError::Forbidden("Failed to retrieve drafts")),
    }
}

// Get single draft
#[get("/recipe_drafts/{id}")]
pub async fn get_recipe_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_recipe_draft(&mut conn, id.into_inner()) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Update draft
#[post("/recipe_drafts/{id}")]
pub async fn update_recipe_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    auth_context: AuthContext,
    updated: web::Json<NewRecipeDraft>,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    let mut updated = updated.into_inner();
    updated.submitted_by = auth_context.user_id;

    log::info!("Updating draft ID: {} {}", id, updated.title);

    match update_recipe_draft(&mut conn, id.into_inner(), &updated) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Delete draft
#[delete("/recipe_drafts/{id}")]
pub async fn delete_recipe_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match delete_recipe_draft(&mut conn, id.into_inner()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Submit draft
#[post("/recipe_drafts/{id}/submit")]
pub async fn submit_draft_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();

    log::info!("Submitting draft ID: {}", id);
    match submit_draft(&mut conn, id.into_inner()) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Request changes
#[post("/recipe_drafts/{id}/request_changes")]
pub async fn request_changes_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    info: web::Json<(i32, String)>, // (reviewer_id, notes)
) -> impl Responder {
    let (reviewer_id, notes) = info.into_inner();
    let mut conn = data.db_pool.get().unwrap();
    match request_changes(&mut conn, id.into_inner(), reviewer_id, &notes) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// curl 'https://dev.revillagesociety.org/api/recipe_drafts/4/approve' \
//   -X 'POST' \
//   -H 'accept: */*' \
//   -H 'accept-language: en-US,en;q=0.9' \
//   -H 'content-length: 0' \
//   -b 'revillage_session=tXa%2FcDO9U5fGpWh1vAdcUO0mWbDbHWWC5oUCQ%2F71KGg9Xi0K8hRi2wbW3PE%3D' \
//   -H 'origin: https://dev.revillagesociety.org' \
//   -H 'priority: u=1, i' \
//   -H 'referer: https://dev.revillagesociety.org/recipes/admin/' \
//   -H 'sec-ch-ua: "Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24"' \
//   -H 'sec-ch-ua-mobile: ?0' \
//   -H 'sec-ch-ua-platform: "macOS"' \
//   -H 'sec-fetch-dest: empty' \
//   -H 'sec-fetch-mode: cors' \
//   -H 'sec-fetch-site: same-origin' \
//   -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36'

// Approve draft
#[post("/recipe_drafts/{id}/approve")]
pub async fn approve_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    auth_context: AuthContext,
    //reviewer_id: web::Json<i32>,
) -> Result<HttpResponse, AuthError> {
    require_role(&auth_context.roles, &[MemberRole::Admin, MemberRole::Reviewer])?;
    let mut conn = data.db_pool.get().unwrap();
    let reviewer_id = auth_context.user_id; // TODO: get from AuthenticatedUser
    match approve_draft(&mut conn, id.into_inner(), reviewer_id) {
        Ok(draft) => Ok(HttpResponse::Ok().json(draft)),
        Err(_) => Err(AuthError::Forbidden("Approval Failed")),
    }
}


// Approve draft
#[post("/recipe_drafts/{id}/deploy")]
pub async fn deploy_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    auth_context: AuthContext,
    //reviewer_id: web::Json<i32>,
) -> Result<HttpResponse, AuthError> {
    require_role(&auth_context.roles, &[MemberRole::Admin, MemberRole::Reviewer])?;
    let mut conn = data.db_pool.get().unwrap();
    let reviewer_id = auth_context.user_id; // TODO: get from AuthenticatedUser
    match deploy_draft(&mut conn, id.into_inner(), reviewer_id) {
        Ok(draft) => Ok(HttpResponse::Ok().json(draft)),
        Err(_) => Err(AuthError::Forbidden("Approval Failed")),
    }
}

fn generate_frontmatter(draft: &RecipeDraft) -> String {
    // Deserialize dietary JSON string into Vec<String>
    let dietary_vec: Vec<String> = draft
        .dietary
        .as_ref()
        .map(|d| serde_json::from_str(d).unwrap_or_default())
        .unwrap_or_default();

    let dietary_yaml = if dietary_vec.is_empty() {
        "[]".to_string()
    } else {
        dietary_vec
            .iter()
            .map(|s| format!(r#""{}""#, s))
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        r#"---
title: "{}"
description: "{}"
tags: [recipe,{}]
layout: "recipe"
author: "{}"
prep_time: {}
cook_time: {}
total_time: {}
servings: {}
difficulty: "{}"
dietary: [{}]
---

{}"#,
        draft.title,
        draft.description,
        draft.tags,
        draft.author,
        draft.prep_time.unwrap_or(0),
        draft.cook_time.unwrap_or(0),
        draft.total_time.unwrap_or(0),
        draft.servings.unwrap_or(0),
        draft.difficulty.clone().unwrap_or_default(),
        dietary_yaml,
        draft.body_md
    )
}

#[get("/recipe_drafts/{id}/md")]
pub async fn get_recipe_draft_md_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_recipe_draft(&mut conn, id.into_inner()) {
        Ok(draft) => {
            let frontmatter = generate_frontmatter(&draft);
            let md_content = format!("{}{}", frontmatter, draft.body_md);
            HttpResponse::Ok()
                .content_type("text/markdown")
                .body(md_content)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


#[post("/recipe_drafts/bulk/approve")]
pub async fn bulk_approve(
    data: web::Data<crate::AppState>,
    auth_context: AuthContext,
    ids: web::Json<Vec<i32>>,
) ->  Result<HttpResponse, AuthError> {

    require_role(&auth_context.roles, &[MemberRole::Admin, MemberRole::Reviewer])?;

    let mut conn = data.db_pool.get().unwrap();
    match approve_drafts(&mut conn, &ids, auth_context.user_id) {
        Ok(_) => (),
        Err(_) => return Err(AuthError::Forbidden("Failed to approve drafts")),
    };
    Ok(HttpResponse::Ok().finish())
        
    
}


// Configure services
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_recipe_draft_api)
        .service(get_recipe_drafts_api)
        .service(bulk_approve)
        .service(get_recipe_draft_api)
        .service(update_recipe_draft_api)
        .service(delete_recipe_draft_api)
        .service(submit_draft_api)
        .service(request_changes_api)
        .service(approve_draft_api)
        .service(get_recipe_draft_md_api)
        .service(deploy_draft_api)
        
        ;
}
