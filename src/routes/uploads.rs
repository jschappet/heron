use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, Responder, Scope, get, post, web};
use futures_util::StreamExt as _;
use serde_json::json;
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use crate::routes::register;
use crate::types::method::Method;

use diesel::prelude::*;

use std::collections::HashSet;

use crate::app_state::AppState;
use crate::models::offers::Offer;
use crate::schema::{offers, users};
use crate::types::JsonField;
use crate::models::users::{PublicUser, User};
use crate::validator::AuthContext; // Your Diesel schema

// #[get("/cleanup_unreferenced")] // @audit-ignore
async fn cleanup_unreferenced(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, Error> {
    let conn = &mut data.db_pool.get().expect("Database connection failed");

    // 1️⃣ Collect referenced images from offers.details JSON
    let all_offers: Vec<Offer> = offers::table
        .load::<Offer>(conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let mut referenced = HashSet::new();
    for offer in all_offers {
        let details: &JsonField = &offer.details; // JsonField wrapper
        if let Some(images) = details.0.get("images").and_then(|v| v.as_array()) {
            for img in images {
                if let Some(url) = img.as_str() {
                    if let Some(filename) = Path::new(url).file_name() {
                        referenced.insert(filename.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    // 1️⃣ Collect referenced images from offers.details JSON
    let all_users: Vec<User> = users::table
        .load::<User>(conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let mut referenced = HashSet::new();
    for user in all_users {
        let public_profile = PublicUser::from(user);
        if let Some(image) = public_profile.image {
            log::info!("Found referenced image: {}", image);
            referenced.insert(image);
        }
    }

    // 2️⃣ Scan upload directory
    let upload_dir = data.settings.web_config.upload_dir.clone();
    let mut deleted_files = vec![];
    for entry in
        fs::read_dir(upload_dir).map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    {
        let entry = entry.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if !referenced.contains(filename) {
                    deleted_files.push(filename.to_string());

                    // Delete only if not dry-run
                    if query.get("dry_run").map(|v| v == "true").unwrap_or(false) == false {
                        if let Err(e) = fs::remove_file(&path) {
                            eprintln!("Failed to delete {}: {:?}", filename, e);
                        } else {
                            println!("Deleted {}", filename);
                        }
                    } else {
                        println!("Dry-run: would delete {}", filename);
                    }
                }
            }
        }
    }

    let dry_run = query.get("dry_run").map(|v| v == "true").unwrap_or(false);
    let message = if dry_run {
        format!(
            "Dry-run mode: {} unreferenced files found",
            deleted_files.len()
        )
    } else {
        format!("Deleted {} unreferenced files", deleted_files.len())
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "deleted_files": deleted_files,
        "dry_run": dry_run,
        "message": message
    })))
}

// #[post("")]
async fn upload(
    mut payload: Multipart,
    data: web::Data<AppState>,
    _user: AuthContext,
) -> Result<impl Responder, Error> {
    // Ensure the upload directory exists
    let upload_dir = data.settings.web_config.upload_dir.clone();
    if let Err(e) = fs::create_dir_all(upload_dir.clone()) {
        eprintln!("Failed to create upload dir: {:?}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"error": "Server error"})));
    }

    // Process the multipart form
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error reading multipart field: {:?}", e);
                return Ok(HttpResponse::BadRequest().json(json!({"error": "Invalid upload"})));
            }
        };

        // Extract filename and extension
        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .as_ref()
            .and_then(|cd| cd.get_filename())
            .unwrap_or("upload.bin")
            .to_string();

        let ext = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");

        // Generate unique name
        let unique_name = format!("{}.{}", Uuid::new_v4(), ext);
        let filepath = format!("{}/{}", upload_dir, unique_name);

        // Write file to disk
        let mut f = tokio::fs::File::create(&filepath).await?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f.write_all(&data).await?;
        }
        let image_site_path = data.settings.web_config.image_site_path.clone();
        let public_url = format!("{}/{}", image_site_path, unique_name);
        println!("Uploaded file saved as {}", public_url);

        return Ok(HttpResponse::Ok().json(json!({ "url": public_url })));
    }

    Ok(HttpResponse::BadRequest().json(json!({"error": "No file found"})))
}

pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
    .service(register(
            "upload",
            Method::POST,
            &full_path,
            "",
            upload,
            crate::types::MemberRole::Member,
        ))

        // cleanup unused files (admin only)
        .service(register(
            "cleanup",
            Method::GET,
            &full_path,
            "/cleanup_unreferenced",
            cleanup_unreferenced,
            crate::types::MemberRole::Admin,
        ))
}
// .service(upload)
//     .service(cleanup_unreferenced)