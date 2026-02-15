use crate::AppState;
use crate::errors::app_error::AppError;
use crate::errors::auth_error::AuthError;
use crate::middleware::host_utils::require_host_id;
use crate::models::drafts::*;
use crate::types::{DocType, DraftStatus, FrontendSchema, MemberRole, load_frontend_schema};
use crate::validator::{ AuthContext, has_role, require_role, require_role_for_host};
use actix_web::{HttpRequest, HttpResponse, Responder, Scope, delete, get, post, web};

use serde::Deserialize;

use once_cell::sync::Lazy;
use std::sync::Arc;

static FRONTEND_SCHEMA: Lazy<Arc<FrontendSchema>> = Lazy::new(|| {
    let schema = load_frontend_schema("./doc_schema.json");
    match schema {
        Ok(good) => Arc::new(good),
        Err(e) => {
            log::error!("Schema load failed: {:?}", e);
            Arc::new(FrontendSchema::default())
        }
    }
});

#[get("/doc_schema")]
async fn ping() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(&*FRONTEND_SCHEMA))
}

// Create or save draft
#[post("")]
pub async fn create_draft_api(
    data: web::Data<AppState>,
    new: web::Json<NewDraft>,
    auth_context: AuthContext,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    let mut new = new.into_inner();
    new.submitted_by = Some(auth_context.user_id);

    match create_draft(&mut conn, &new) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        //.finish(),
    }
}

use serde::Deserializer;
use serde::de::IntoDeserializer;

fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => Ok(Some(T::deserialize(s.into_deserializer())?)),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct DraftQuery {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub status: Option<DraftStatus>,
    pub docType: Option<String>,
    pub author: Option<String>,
    pub dateFrom: Option<String>,
    pub dateTo: Option<String>,
}

use serde::Serialize;

#[derive(Serialize)]
pub struct DraftsMeta {
    pub total: usize,
    pub doc_types: Vec<&'static str>,
    pub statuses: Vec<String>,
}

#[derive(Serialize)]
pub struct DraftsResponse<T> {
    pub meta: DraftsMeta,
    pub drafts: Vec<T>,
}

#[get("")]
pub async fn get_drafts_api(
    data: web::Data<AppState>,
    admin_context: AuthContext,
    req: HttpRequest,
    query: web::Query<DraftQuery>,
) -> Result<HttpResponse, AuthError> {
    let mut conn = data.db_pool.get().unwrap();
    let host_id = require_host_id(&req).await.unwrap(); // safe fallback exists

    // Check if the user has Admin or Reviewer role for this host
    let is_privileged = admin_context.memberships.iter().any(|m| {
        m.host_id == host_id && matches!(m.role, MemberRole::Admin | MemberRole::Reviewer)
    });

    // Only filter by user if not privileged
    let user_filter = if is_privileged {
        None
    } else {
        Some(admin_context.user_id)
    };

    let mut filter = DraftFilter::from(query.0);
    filter.submitted_by = user_filter;

    let drafts = get_drafts_filtered(&mut conn, filter)
        .map_err(|_| AuthError::Forbidden("Failed to retrieve drafts"))?;

    let meta = DraftsMeta {
        total: drafts.len(),
        doc_types: DocType::list(),
        statuses: drafts
            .iter()
            .map(|d| d.status.to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect(),
    };

    let response = DraftsResponse { meta, drafts };

    Ok(HttpResponse::Ok().json(response))
}

// Get single draft
#[get("/{id}")]
pub async fn get_draft_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_draft(&mut conn, id.into_inner()) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Update draft
#[post("/{id}")]
pub async fn update_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    auth_context: AuthContext,
    updated: web::Json<NewDraft>,
) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    let mut updated = updated.into_inner();
    updated.submitted_by = Some(auth_context.user_id);

    log::info!("Updating draft ID: {} {}", id, updated.title);

    match update_draft(&mut conn, id.into_inner(), &updated) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Delete draft
#[delete("/{id}")]
pub async fn delete_draft_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match delete_draft(&mut conn, id.into_inner()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Submit draft
#[post("/{id}/submit")]
pub async fn submit_draft_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();

    log::info!("Submitting draft ID: {}", id);
    match submit_draft(&mut conn, id.into_inner()) {
        Ok(draft) => HttpResponse::Ok().json(draft),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Request changes
#[post("/{id}/request_changes")]
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

// curl 'https://dev.revillagesociety.org/api/drafts/4/approve' \
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
#[post("/{id}/approve")]
pub async fn approve_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    admin_context: AuthContext,
    req: HttpRequest,
    //reviewer_id: web::Json<i32>,
) -> Result<HttpResponse, AuthError> {
       let incoming_host_id = require_host_id(&req).await.unwrap(); // safe because fallback exists

    require_role_for_host(
        &admin_context,
        incoming_host_id,
        &[MemberRole::Admin, MemberRole::Reviewer]
    )?;
    let mut conn = data.db_pool.get().unwrap();
    let reviewer_id = admin_context.user_id; // TODO: get from AuthenticatedUser
    match approve_draft(&mut conn, id.into_inner(), reviewer_id) {
        Ok(draft) => Ok(HttpResponse::Ok().json(draft)),
        Err(_) => Err(AuthError::Forbidden("Approval Failed")),
    }
}

// Approve draft
#[post("/{id}/deploy")]
pub async fn deploy_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    admin_context: AuthContext,
    req:HttpRequest,
    //reviewer_id: web::Json<i32>,
) -> Result<HttpResponse, AuthError> {
       let incoming_host_id = require_host_id(&req).await.unwrap(); // safe because fallback exists

    require_role_for_host(
        &admin_context,
        incoming_host_id,
        &[MemberRole::Admin, MemberRole::Reviewer]
    )?;
    let mut conn = data.db_pool.get().unwrap();
    let reviewer_id = admin_context.user_id; // TODO: get from AuthenticatedUser
    match deploy_draft(&mut conn, id.into_inner(), reviewer_id) {
        Ok(draft) => Ok(HttpResponse::Ok().json(draft)),
        Err(_) => Err(AuthError::Forbidden("Approval Failed")),
    }
}

fn opt_str(s: &Option<String>) -> &str {
    s.as_deref().unwrap_or("")
}

fn _get_value_from_map(key: &str, map: &serde_json::Map<String, Value>) -> String {
    if let Some(v) = map.get(key) {
        format!("{}: {}", key, v)
    } else {
        String::from(r#"key_error: "{} not found"", {}"#)
    }

}

fn format_frontmatter_value(key: &str, v: &serde_json::Value, _field_type: &str) -> Option<String> {
    match v {
        serde_json::Value::String(s) => {
            Some(format!(r#"{key}: "{}""#, s))
        }
        serde_json::Value::Number(n) => {
            Some(format!(r#"{key}: {}"#, n))
        }
        serde_json::Value::Bool(b) => {
            Some(format!(r#"{key}: {}"#, b))
        }
        serde_json::Value::Array(arr) => {
            let items = arr
                .iter()
                .filter_map(|x| x.as_str())
                .map(|s| format!(r#""{}""#, s))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("{key}: [{items}]"))
        }
        _ => None,
    }
}

fn format_frontmatter_scalar(key: &str, v: &str) -> String {
    format!(r#"{key}: "{}""#, v)
}


use serde_json::Value;

fn generate_frontmatter(draft: &Draft) -> String {
    // Parse meta JSON if present
    let meta: Value = draft
        .meta
        .as_ref()
        .map(|m| m.0.clone())
        .unwrap_or(Value::Null);
    log::debug!("Meta: {:?}", meta);
    // Determine frontmatter layout and fields based on doc_type
    let layout = draft.doc_type.value();

    // Base fields: title, description, author
    let mut fm_lines = vec![
        format!(r#"title: "{}""#, draft.title),
        format!(r#"description: "{}""#, opt_str(&draft.description)),
        format!(r#"author: "{}""#, opt_str(&draft.author)),
    ];

    // Tags
    let tags_yaml = match &draft.tags {
        Some(tags_str) => {
            // Try to parse JSON array first
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(tags_str) {
                arr.iter()
                    .map(|s| format!(r#""{}""#, s))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                // Fallback: split by comma
                tags_str
                    .split(',')
                    .map(|s| format!(r#""{}""#, s.trim()))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        }
        None => "".to_string(),
    };
    fm_lines.push(format!("tags: [{}]", tags_yaml));

    if let Some(doc_schema) = FRONTEND_SCHEMA.types.get(&draft.doc_type) {
        for field in &doc_schema.fields {
            // Skip non-displayed fields
            if field.display == Some(false) {
                continue;
            }

            // Skip markdown body field
            if field.field_type == "markdown" || field.key == "body_md" {
                continue;
            }

            match field.storage.as_deref() {
                Some("meta") => {
                    if let Value::Object(map) = &meta {
                        if let Some(v) = map.get(&field.key) {
                            if let Some(line) =
                                format_frontmatter_value(&field.key, v, &field.field_type)
                            {
                                fm_lines.push(line);
                            }
                        }
                    }
                }
                // Column or default storage: pull from draft
                _ => {
                    if let Some(v) = draft.get_field_value(&field.key) {
                        fm_lines.push(format_frontmatter_scalar(&field.key, &v));
                    }
                }
            }
        }
    } else {
        fm_lines.push(format!(r#"doc_type: "{:?} (no schema)""#, draft.doc_type));
    }

    // Layout
    fm_lines.push(format!(r#"layout: "{}""#, layout));

    // Join lines into YAML frontmatter
    let frontmatter = format!("---\n{}\n---\n\n", fm_lines.join("\n"));
    frontmatter
}

#[get("/{id}/md")]
pub async fn get_draft_md_api(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let mut conn = data.db_pool.get().unwrap();
    match get_draft(&mut conn, id.into_inner()) {
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

#[post("/bulk/approve")]
pub async fn bulk_approve(
    data: web::Data<crate::AppState>,
    admin_context: AuthContext,
    ids: web::Json<Vec<i32>>,
    req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    let incoming_host_id = require_host_id(&req).await.unwrap(); // safe because fallback exists

    require_role_for_host(
        &admin_context,
        incoming_host_id,
        &[MemberRole::Admin, MemberRole::Reviewer]
    )?;

    let mut conn = data.db_pool.get().unwrap();
    match approve_drafts(&mut conn, &ids, admin_context.user_id) {
        Ok(_) => (),
        Err(_) => return Err(AuthError::Forbidden("Failed to approve drafts")),
    };
    Ok(HttpResponse::Ok().finish())
}

pub fn scope() -> Scope {
    web::scope("")
        .service(ping)
        .service(create_draft_api)
        .service(get_drafts_api)
        .service(bulk_approve)
        .service(get_draft_api)
        .service(update_draft_api)
        .service(delete_draft_api)
        .service(submit_draft_api)
        .service(request_changes_api)
        .service(approve_draft_api)
        .service(get_draft_md_api)
        .service(deploy_draft_api)
}
