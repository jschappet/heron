use crate::app_state::AppState;
use crate::domains::draft_domain::{self, DraftDomain};
use crate::domains::ledger_domain::LedgerDomain;
use crate::errors::app_error::AppError;
use crate::errors::auth_error::AuthError;
use crate::middleware::host::{HostContext};
use crate::models::drafts::*;
use crate::types::{DocType, DraftStatus, FrontendSchema, JsonField, MemberRole, load_frontend_schema};
use crate::validator::{AuthContext, require_role_for_host};
use actix_web::{HttpResponse, Scope, web};
use diesel::Connection;

use crate::routes::{register, RoutePath};
use crate::types::method::Method;

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

fn contributor(auth: &AuthContext, host_id: i32) -> Result<(), AuthError> {
    require_role_for_host(auth, host_id, &[MemberRole::Member, MemberRole::Reviewer, MemberRole::Admin])
}

fn review_access(auth: &AuthContext, host_id: i32) -> Result<(), AuthError> {
    require_role_for_host(auth, host_id, &[MemberRole::Reviewer, MemberRole::Admin])
}

fn check_draft(draft: &Draft, auth: &AuthContext, host_id: i32, editing: bool) -> Result<(), AppError> {
    contributor(auth, host_id)?;
    if draft.host_id != host_id || draft.status == "deleted" {
        return Err(AppError::NotFound("Content not found".into()));
    }
    let privileged = review_access(auth, host_id).is_ok();
    if draft.submitted_by != auth.user_id && !privileged {
        return Err(AuthError::Forbidden("This content belongs to another contributor").into());
    }
    if editing && !privileged && !matches!(draft.status.as_str(), "draft" | "changes_requested") {
        return Err(AuthError::Forbidden("Only drafts and changes requested can be edited").into());
    }
    Ok(())
}

fn checked_draft(conn: &mut diesel::SqliteConnection, id: i32, auth: &AuthContext, host_id: i32, editing: bool) -> Result<Draft, AppError> {
    let draft = get_draft(conn, id)?;
    check_draft(&draft, auth, host_id, editing)?;
    Ok(draft)
}

// Workflow and ownership fields are always server controlled.
fn clean_payload(new: &mut NewDraft, owner: i32, host_id: i32, status: &str) -> Result<(), AppError> {
    let kind: DocType = serde_json::from_value(serde_json::json!(new.doc_type))
        .map_err(|_| AppError::BadRequest("Unknown content type".into()))?;
    if !FRONTEND_SCHEMA.types.contains_key(&kind) {
        return Err(AppError::BadRequest("Content type is not configured".into()));
    }
    new.submitted_by = Some(owner);
    new.host_id = host_id;
    new.status = Some(status.into());
    new.submitted_at = None;
    new.reviewed_by = None;
    new.reviewed_at = None;
    new.review_notes = None;
    Ok(())
}

async fn get_doc_schema(auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    contributor(&auth, host.0.id)?;
    Ok(HttpResponse::Ok().json(&*FRONTEND_SCHEMA))
}

pub async fn create_draft_api(
    new: web::Json<NewDraft>, auth_context: AuthContext, host: HostContext,
    domain: web::Data<DraftDomain>,
) -> Result<HttpResponse, AppError> {
    contributor(&auth_context, host.0.id)?;
    let mut new = new.into_inner();
    clean_payload(&mut new, auth_context.user_id, host.0.id, "draft")?;
    Ok(HttpResponse::Ok().json(domain.create_draft(&new)?))
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

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct DraftQuery {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub status: Option<DraftStatus>,
    pub docType: Option<String>,
    pub author: Option<String>,
    pub dateFrom: Option<String>,
    pub dateTo: Option<String>,
    #[serde(default)]
    pub host_id: i32,
    #[serde(default)]
    pub mine: bool,
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

//#[get("")]
pub async fn get_drafts_api(
    data: web::Data<AppState>,
    admin_context: AuthContext,
    host: HostContext,
    //req: HttpRequest,
    query: web::Query<DraftQuery>,
) -> Result<HttpResponse, AuthError> {
    log::debug!("Draft Query: {:?}", query);

    let mut conn = data.db_pool.get().unwrap();
    let host_id = host.0.id;
    contributor(&admin_context, host_id)?;
    let mut query = query.0;
    query.host_id = host_id;

    log::debug!("Draft Query: {:?}", query);

    // Check if the user has Admin or Reviewer role for this host
    let is_privileged = admin_context.memberships.iter().any(|m| {
        m.host_id == host_id && matches!(m.role, MemberRole::Admin | MemberRole::Reviewer)
    });

    // Only filter by user if not privileged
    let user_filter = if is_privileged && !query.mine {
        None
    } else {
        Some(admin_context.user_id)
    };

    let mut filter = DraftFilter::from(query);
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
//#[get("/{id}")]
pub async fn get_draft_api(data: web::Data<AppState>, id: web::Path<i32>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    Ok(HttpResponse::Ok().json(checked_draft(&mut conn, *id, &auth, host.0.id, false)?))
}

pub async fn update_draft_api(data: web::Data<AppState>, id: web::Path<i32>, auth_context: AuthContext,
    updated: web::Json<NewDraft>, host: HostContext) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let mut updated = updated.into_inner();
    let saved = conn.immediate_transaction::<_, AppError, _>(|conn| {
        let draft = checked_draft(conn, *id, &auth_context, host.0.id, true)?;
        clean_payload(&mut updated, draft.submitted_by, host.0.id, &draft.status)?;
        if updated.doc_type != draft.doc_type.value() {
            return Err(AppError::BadRequest("Content type cannot be changed".into()));
        }
        Ok(update_draft(conn, *id, &updated)?)
    })?;
    Ok(HttpResponse::Ok().json(saved))
}

// Bulk soft deletion used by the admin content page.
pub async fn delete_draft_api(
    data: web::Data<AppState>,
    ids: web::Json<BulkIds>,
    auth_context: AuthContext,
    host: HostContext,
) -> Result<HttpResponse, AuthError> {
    require_role_for_host(&auth_context, host.0.id, &[MemberRole::Admin])?;
    let mut conn = data.db_pool.get().map_err(|e| AuthError::Internal(e.to_string()))?;
    let deleted = delete_drafts(&mut conn, &ids.ids, host.0.id)
        .map_err(|e| AuthError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({ "deleted": deleted })))
}

// Preserve the existing single-draft endpoint with the same soft-delete behavior.
pub async fn delete_single_draft_api(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    auth_context: AuthContext,
    host: HostContext,
) -> Result<HttpResponse, AuthError> {
    require_role_for_host(&auth_context, host.0.id, &[MemberRole::Admin])?;
    let mut conn = data.db_pool.get().map_err(|e| AuthError::Internal(e.to_string()))?;
    match delete_drafts(&mut conn, &[id.into_inner()], host.0.id)
        .map_err(|e| AuthError::Internal(e.to_string()))? {
        0 => Ok(HttpResponse::NotFound().finish()),
        _ => Ok(HttpResponse::Ok().finish()),
    }
}

// Submit draft
//#[post("/{id}/submit")]
fn validate_submission(draft: &Draft) -> Result<(), AppError> {
    let schema = FRONTEND_SCHEMA.types.get(&draft.doc_type)
        .ok_or_else(|| AppError::BadRequest("Content type is not configured".into()))?;
    let columns = serde_json::to_value(draft).map_err(|e| AppError::Internal(e.to_string()))?;
    for field in schema.fields.iter().filter(|f| f.required == Some(true)) {
        let mut container = if field.storage.as_deref() == Some("meta") { &columns["meta"] } else { &columns };
        if let Some(path) = &field.path {
            for key in path.split('.').filter(|p| !p.is_empty()) { container = &container[key]; }
        }
        let value = &container[&field.key];
        if value.is_null() || value.as_str().map_or(false, |s| s.trim().is_empty()) || value.as_array().map_or(false, |a| a.is_empty()) {
            return Err(AppError::BadRequest(format!("{} is required before submission", field.label)));
        }
    }
    Ok(())
}

pub async fn submit_draft_api(data: web::Data<AppState>, id: web::Path<i32>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let saved = conn.immediate_transaction::<_, AppError, _>(|conn| {
        let draft = checked_draft(conn, *id, &auth, host.0.id, true)?;
        if !matches!(draft.status.as_str(), "draft" | "changes_requested") {
            return Err(AppError::BadRequest("Content is already submitted or approved".into()));
        }
        validate_submission(&draft)?;
        Ok(submit_draft(conn, *id)?)
    })?;
    Ok(HttpResponse::Ok().json(saved))
}

pub async fn request_changes_api(data: web::Data<AppState>, id: web::Path<i32>, info: web::Json<(i32, String)>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    review_access(&auth, host.0.id)?;
    let mut conn = data.db_conn()?;
    checked_draft(&mut conn, *id, &auth, host.0.id, false)?;
    Ok(HttpResponse::Ok().json(request_changes(&mut conn, *id, auth.user_id, &info.1)?))
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
//#[post("/{id}/approve")]
pub async fn approve_draft_api(data: web::Data<AppState>, id: web::Path<i32>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    review_access(&auth, host.0.id)?;
    let mut conn = data.db_conn()?;
    let saved = conn.immediate_transaction::<_, AppError, _>(|conn| {
        let draft = checked_draft(conn, *id, &auth, host.0.id, false)?;
        if draft.status != "submitted" { return Err(AppError::BadRequest("Content must be submitted first".into())); }
        Ok(approve_draft(conn, *id, auth.user_id)?)
    })?;
    Ok(HttpResponse::Ok().json(saved))
}

// Approve draft
//#[post("/{id}/deploy")]
pub async fn deploy_draft_api(data: web::Data<AppState>, id: web::Path<i32>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    review_access(&auth, host.0.id)?;
    let mut conn = data.db_conn()?;
    let saved = conn.immediate_transaction::<_, AppError, _>(|conn| {
        let draft = checked_draft(conn, *id, &auth, host.0.id, false)?;
        if draft.status != "approved" { return Err(AppError::BadRequest("Content must be approved first".into())); }
        Ok(deploy_draft(conn, *id, auth.user_id)?)
    })?;
    Ok(HttpResponse::Ok().json(saved))
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
        serde_json::Value::String(s) => Some(format!(r#"{key}: "{}""#, s)),
        serde_json::Value::Number(n) => Some(format!(r#"{key}: {}"#, n)),
        serde_json::Value::Bool(b) => Some(format!(r#"{key}: {}"#, b)),
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

use serde_json::{Value, json};
use serde_yaml;
use std::collections::BTreeMap;

fn generate_frontmatter(draft: &Draft) -> String {
    // Start with a BTreeMap to preserve order of top-level fields
    let mut root: BTreeMap<String, Value> = BTreeMap::new();
    let author = draft.author.clone().unwrap_or_default();

    // Top-level fields (columns)
    root.insert("title".to_string(), Value::String(draft.title.clone()));
    root.insert("author".to_string(), Value::String(author));
    root.insert(
        "doc_type".to_string(),
        Value::String(draft.doc_type.value().to_string()),
    );

    // Optional description
    if let Some(desc) = &draft.description {
        root.insert("description".to_string(), Value::String(desc.clone()));
    }

    let tags_yaml = match &draft.tags {
        Some(tags_str) => {
            // Try to parse JSON array first
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(tags_str) {
                arr.iter().map(|s| Value::String(s.clone())).collect()
            } else {
                // Fallback: split by comma
                tags_str
                    .split(',')
                    .map(|s| Value::String(s.trim().to_string()))
                    .collect()
            }
        }
        None => Vec::new(),
    };
    root.insert("tags".to_string(), Value::Array(tags_yaml));

    // Merge nested meta
    if let Some(meta) = &draft.meta {
        if let Value::Object(map) = &meta.0 {
            for (k, v) in map {
                //let key_string = k.clone();     // owned String

                root.insert(k.to_string(), v.clone());
            }
        }
    }

    // Serialize whole structure to YAML
    let yaml_body = serde_yaml::to_string(&root).expect("Failed to serialize YAML");

    // Wrap in frontmatter ---
    format!("---\n{}---\n\n", yaml_body)
}

//use serde_json::Value;

fn _generate_frontmatter_old(draft: &Draft) -> String {
    // Parse meta JSON if present
    let meta: Value = draft
        .meta
        .as_ref()
        .map(|m| m.0.clone())
        .unwrap_or(Value::Null);
    log::debug!("Meta: {:?}", meta);
    // Determine frontmatter layout and fields based on doc_type
    //let layout = draft.doc_type.value();

    // Base fields: title, description, author
    let mut fm_lines = vec![
        //format!(r#"title: "{}""#, draft.title),
        //format!(r#"description: "{}""#, opt_str(&draft.description)),
        //format!(r#"author: "{}""#, opt_str(&draft.author)),
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
    //fm_lines.push(format!(r#"layout: "{}""#, layout));

    // Join lines into YAML frontmatter
    let frontmatter = format!("---\n{}\n---\n\n", fm_lines.join("\n"));
    frontmatter
}

//#[get("/{id}/md")]
pub async fn get_draft_md_api(data: web::Data<AppState>, id: web::Path<i32>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    let mut conn = data.db_conn()?;
    let draft = checked_draft(&mut conn, *id, &auth, host.0.id, false)?;
    Ok(HttpResponse::Ok().content_type("text/markdown").body(format!("{}{}", generate_frontmatter(&draft), draft.body_md)))
}
#[derive(Deserialize, Debug)]
pub struct BulkIds {
    pub ids: Vec<i32>,
}

//#[post("/bulk/approve")]
pub async fn bulk_approve(data: web::Data<AppState>, auth: AuthContext, ids: web::Json<BulkIds>, host: HostContext) -> Result<HttpResponse, AppError> {
    review_access(&auth, host.0.id)?;
    let mut conn = data.db_conn()?;
    conn.immediate_transaction::<_, AppError, _>(|conn| {
        for id in &ids.ids {
            let draft = checked_draft(conn, *id, &auth, host.0.id, false)?;
            if draft.status != "submitted" { return Err(AppError::BadRequest("Only submitted content can be approved".into())); }
        }
        approve_drafts(conn, &ids.ids, auth.user_id)?;
        Ok(())
    })?;
    Ok(HttpResponse::Ok().finish())
}


// Contributor routes bypass the legacy admin-only URL scope. Legacy URLs remain available.
pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register("draft_create_public", Method::POST, path.as_str(), "new", create_draft_api, MemberRole::Member))
        .service(register("content_create", Method::POST, path.as_str(), "create", create_draft_api, MemberRole::Member))
        .service(register("content_list", Method::GET, path.as_str(), "list", get_drafts_api, MemberRole::Member))
        .service(register("content_schema", Method::GET, path.as_str(), "doc_schema", get_doc_schema, MemberRole::Member))
        .service(register("content_get", Method::GET, path.as_str(), "get/{id}", get_draft_api, MemberRole::Member))
        .service(register("content_update", Method::POST, path.as_str(), "update/{id}", update_draft_api, MemberRole::Member))
        .service(register("content_submit", Method::POST, path.as_str(), "{id}/submit", submit_draft_api, MemberRole::Member))
        .service(register("content_approve", Method::POST, path.as_str(), "{id}/approve", approve_draft_api, MemberRole::Reviewer))
}

pub fn admin_scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register(
            "delete_draft_api",
            Method::POST,
            path.as_str(),
            "bulk/delete",
            delete_draft_api,
            MemberRole::Admin,
        ))
        .service(register(
            "draft_bulk_approve",
            Method::POST,
            path.as_str(),
            "bulk/approve",
            bulk_approve,
            MemberRole::Admin,
        )) // Schema
        .service(register(
            "draft_update",
            Method::POST,
            path.as_str(),
            "update/{id}",
            update_draft_api,
            MemberRole::Member,
        ))
        .service(register(
            "draft_doc_schema",
            Method::GET,
            path.as_str(),
            "doc_schema",
            get_doc_schema,
            MemberRole::Member,
        ))
        // Create & List
        .service(register(
            "draft_create",
            Method::POST,
            path.as_str(),
            "new",
            create_draft_api,
            MemberRole::Member,
        ))
        .service(register(
            "draft_list",
            Method::GET,
            path.as_str(),
            "list",
            get_drafts_api,
            MemberRole::Member,
        ))
        // Single Draft CRUD
        .service(register(
            "draft_get",
            Method::GET,
            path.as_str(),
            "get/{id}",
            get_draft_api,
            MemberRole::Member,
        ))
        .service(register(
            "draft_delete",
            Method::DELETE,
            path.as_str(),
            "{id}",
            delete_single_draft_api,
            MemberRole::Admin,
        ))
        // Workflow Actions
        .service(register(
            "draft_submit",
            Method::POST,
            path.as_str(),
            "{id}/submit",
            submit_draft_api,
            MemberRole::Member,
        ))
        .service(register(
            "draft_request_changes",
            Method::POST,
            path.as_str(),
            "{id}/request_changes",
            request_changes_api,
            MemberRole::Reviewer,
        ))
        // Review / Publish
        .service(register(
            "draft_approve",
            Method::POST,
            path.as_str(),
            "{id}/approve",
            approve_draft_api,
            MemberRole::Reviewer,
        ))
        .service(register(
            "draft_deploy",
            Method::POST,
            path.as_str(),
            "{id}/deploy",
            deploy_draft_api,
            MemberRole::Reviewer,
        ))
        // Markdown Export
        .service(register(
            "draft_markdown",
            Method::GET,
            path.as_str(),
            "{id}/md",
            get_draft_md_api,
            MemberRole::Member,
        ))
    // Bulk Approval
}

// .service(get_doc_schema)
// .service(create_draft_api)
// .service(get_drafts_api)
// .service(bulk_approve)
// .service(get_draft_api)
// .service(update_draft_api)
// .service(delete_draft_api)
// .service(submit_draft_api)
// .service(request_changes_api)
// .service(approve_draft_api)
// .service(get_draft_md_api)
// .service(deploy_draft_api)

#[cfg(test)]
mod deletion_tests {
    use super::*;

    #[actix_web::test]
    async fn bulk_delete_capability_matches_content_page_request() {
        let _scope = admin_scope(&RoutePath::new(&["api", "admin", "drafts"]));
        let routes = crate::routes::routes().lock().unwrap();
        let route = routes.iter().find(|r| r.key == "delete_draft_api").unwrap();
        assert_eq!(route.method, "POST");
        assert_eq!(route.segments.last().unwrap(), "bulk/delete");
        assert_eq!(route.roles, vec![MemberRole::Admin]);
        let body: BulkIds = serde_json::from_value(json!({"ids": [1, 2]})).unwrap();
        assert_eq!(body.ids, vec![1, 2]);
    }
}

#[cfg(test)]
mod research_tests {
    use super::*;

    #[test]
    fn experiment_schema_and_result_links_survive_export() {
        let schema = load_frontend_schema("./doc_schema.json").unwrap();
        assert!(DocType::list().contains(&"experiment"));
        let experiment = schema.types.get(&DocType::Experiment).unwrap();
        assert!(experiment.fields.iter().any(|f| f.key == "experiment_id"));
        let result = schema.types.get(&DocType::Results).unwrap();
        assert!(result.fields.iter().any(|f| f.key == "experiment_id"));
        let draft = Draft {
            id: 1, doc_type: DocType::Results, title: "Field observation".into(),
            description: None, tags: None, author: None,
            meta: Some(JsonField(json!({"experiment_id": "soil", "result_id": "soil-one", "summary": "A useful observation"}))),
            body_md: "Notes".into(), status: "approved".into(), submitted_by: 1,
            submitted_at: None, reviewed_by: None, reviewed_at: None,
            review_notes: None, details: None, host_id: 1,
        };
        let markdown = generate_frontmatter(&draft);
        let yaml = markdown.trim().trim_matches('-').trim();
        let exported: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(exported["experiment_id"].as_str(), Some("soil"));
        assert_eq!(exported["result_id"].as_str(), Some("soil-one"));
        assert_eq!(exported["doc_type"].as_str(), Some("results"));
        assert_eq!(serde_json::from_str::<DocType>("\"experiment\"").unwrap(), DocType::Experiment);
    }
}

#[cfg(test)]
mod content_permissions_tests {
    use super::*;
    use diesel::{Connection, connection::SimpleConnection};
    use crate::validator::MembershipContext;

    fn auth(user_id: i32, host_id: i32, role: MemberRole) -> AuthContext {
        AuthContext { user_id, memberships: vec![MembershipContext {host_id, role}] }
    }
    fn database() -> diesel::SqliteConnection {
        let mut conn = diesel::SqliteConnection::establish(":memory:").unwrap();
        conn.batch_execute(include_str!("../../migrations/2026-01-04-170231_drafts/up.sql")).unwrap();
        conn.batch_execute(include_str!("../../migrations/2026-02-28-153103_addHostIdDraft/up.sql")).unwrap();
        conn.batch_execute("INSERT INTO drafts (id, doc_type, title, body_md, submitted_by, host_id) VALUES
            (1, 'post', 'Own draft', 'Body', 7, 1), (2, 'post', 'Private', '', 8, 1), (3, 'post', 'Other site', '', 7, 2);").unwrap();
        conn
    }
    #[test]
    fn ownership_and_site_are_checked_for_direct_ids_and_export() {
        let mut conn = database();
        let member = auth(7, 1, MemberRole::Member);
        assert!(checked_draft(&mut conn, 1, &member, 1, true).is_ok());
        assert!(checked_draft(&mut conn, 2, &member, 1, false).is_err());
        assert!(checked_draft(&mut conn, 3, &member, 1, false).is_err());
        assert!(checked_draft(&mut conn, 1, &auth(7, 2, MemberRole::Admin), 1, false).is_err());
        assert!(checked_draft(&mut conn, 2, &auth(7, 1, MemberRole::Reviewer), 1, true).is_ok());
        assert!(checked_draft(&mut conn, 3, &auth(7, 1, MemberRole::Admin), 1, false).is_err());
        assert!(review_access(&member, 1).is_err());
    }
    #[test]
    fn contributor_can_read_but_not_edit_submitted_and_approved_work() {
        let mut conn = database();
        let member = auth(7, 1, MemberRole::Member);
        submit_draft(&mut conn, 1).unwrap();
        assert!(checked_draft(&mut conn, 1, &member, 1, false).is_ok());
        assert!(checked_draft(&mut conn, 1, &member, 1, true).is_err());
        approve_draft(&mut conn, 1, 9).unwrap();
        assert!(checked_draft(&mut conn, 1, &member, 1, true).is_err());
        request_changes(&mut conn, 1, 9, "Revise").unwrap();
        assert!(checked_draft(&mut conn, 1, &member, 1, true).is_ok());
        delete_drafts(&mut conn, &[1], 1).unwrap();
        assert!(checked_draft(&mut conn, 1, &member, 1, false).is_err());
    }
    #[test]
    fn submission_requires_configured_fields_but_allows_incomplete_drafts() {
        let mut conn = database();
        let mut draft = get_draft(&mut conn, 1).unwrap();
        assert!(validate_submission(&draft).is_ok()); // Post description/author are optional.
        draft.body_md.clear();
        assert!(validate_submission(&draft).is_err());
        draft.body_md = "Body".into();
        draft.title = "  ".into();
        assert!(validate_submission(&draft).is_err());
        assert!(checked_draft(&mut conn, 2, &auth(8, 1, MemberRole::Member), 1, true).is_ok());
    }

    #[test]
    fn save_cannot_forge_approval_owner_or_host() {
        let mut conn = database();
        let mut payload: NewDraft = serde_json::from_value(json!({"doc_type":"post", "title":"Edited", "body_md":"Body", "status":"approved", "submitted_by":99, "host_id":2, "reviewed_by":99})).unwrap();
        clean_payload(&mut payload, 7, 1, "draft").unwrap();
        let saved = update_draft(&mut conn, 1, &payload).unwrap();
        assert_eq!((saved.host_id, saved.submitted_by, saved.status.as_str(), saved.reviewed_by), (1, 7, "draft", None));
    }
}
