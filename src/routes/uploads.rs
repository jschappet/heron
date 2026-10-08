use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, Responder, Scope, web};
use futures_util::StreamExt as _;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use crate::routes::{register, RoutePath};
use crate::types::method::Method;

use actix_files::NamedFile;
use serde::{Deserialize, Serialize};

use diesel::prelude::*;

use std::collections::HashSet;

use crate::app_state::AppState;
use crate::models::offers::Offer;
use crate::schema::{offers, users};
use crate::types::JsonField;
use crate::models::users::{PublicUser, User};
use crate::validator::AuthContext; // Your Diesel schema

const AUDIO_EXTENSIONS: &[&str] = &["m4a", "mp3", "wav", "aac", "mp4", "m4z"];


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
        log::info!("Uploaded file saved as {}", filepath);

        return Ok(HttpResponse::Ok().json(json!({ "url": public_url })));
    }

    Ok(HttpResponse::BadRequest().json(json!({"error": "No file found"})))
}


// #[post("/api/transcription/upload")]
async fn upload_audio(
    mut payload: Multipart,
    data: web::Data<AppState>,
    //_user: AuthContext,
) -> Result<impl Responder, Error> {
    const MAX_AUDIO_BYTES: u64 = 512 * 1024 * 1024;

    let upload_dir = PathBuf::from(
        data.settings.web_config.upload_dir.clone()
    )
    .join("transcription-inbox");

    tokio::fs::create_dir_all(&upload_dir).await?;

    //let allowed_extensions = ["m4a", "mp3", "wav", "aac", "mp4", "m4z"];
    let mut saved_file = None;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(_) => {
                return Ok(HttpResponse::BadRequest()
                    .json(json!({"error": "Invalid multipart upload"})));
            }
        };

        let content_disposition = field.content_disposition().cloned();

        let field_name = content_disposition
            .as_ref()
            .and_then(|cd| cd.get_name());

        // The iPhone Shortcut should send the file in a field named "file".
        if field_name != Some("file") {
            while field.next().await.transpose()?.is_some() {}
            continue;
        }

        let original_filename = content_disposition
            .as_ref()
            .and_then(|cd| cd.get_filename())
            .unwrap_or("recording.m4a");

        let extension = Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("m4a")
            .to_ascii_lowercase();

        if !AUDIO_EXTENSIONS.contains(&extension.as_str()) {
            return Ok(HttpResponse::UnsupportedMediaType()
                .json(json!({
                    "error": "Unsupported audio format",
                    "allowed": AUDIO_EXTENSIONS
                })));
        }

        let id = Uuid::new_v4().to_string();
        let filename = format!("{}.{}", id, extension);
        let final_path = upload_dir.join(&filename);
        let partial_path = upload_dir.join(format!(".{}.part", id));

        let mut file = tokio::fs::File::create(&partial_path).await?;
        let mut total_bytes: u64 = 0;

        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(_) => {
                    let _ = tokio::fs::remove_file(&partial_path).await;

                    return Ok(HttpResponse::BadRequest()
                        .json(json!({"error": "Upload interrupted"})));
                }
            };

            total_bytes += chunk.len() as u64;

            if total_bytes > MAX_AUDIO_BYTES {
                let _ = tokio::fs::remove_file(&partial_path).await;

                return Ok(HttpResponse::PayloadTooLarge()
                    .json(json!({"error": "Audio file is too large"})));
            }

            file.write_all(&chunk).await?;
        }

        file.flush().await?;

        // Atomic rename means the watcher will only see complete files.
        tokio::fs::rename(&partial_path, &final_path).await?;

        saved_file = Some((id, filename, total_bytes));
        break;
    }

    match saved_file {
        Some((id, filename, size)) => {
            println!(
                "Queued transcription file: {} ({} bytes)",
                filename, size
            );

            Ok(HttpResponse::Ok().json(json!({
                "status": "queued",
                "id": id,
                "filename": filename,
                "size": size
            })))
        }

        None => Ok(HttpResponse::BadRequest()
            .json(json!({"error": "No audio file found"}))),
    }
}




fn transcription_dir(data: &web::Data<AppState>) -> PathBuf {
    PathBuf::from(data.settings.web_config.upload_dir.clone())
        .join("transcription-inbox")
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.contains('/')
        && !id.contains('\\')
        && !id.contains("..")
}

#[derive(Serialize)]
struct AudioJob {
    id: String,
    filename: String,
    download_url: String,
}

//#[get("/transcript/audio")]
async fn next_audio(
    data: web::Data<AppState>,
    //_user: AuthContext,
) -> Result<impl Responder, Error> {
    let directory = transcription_dir(&data);
    tokio::fs::create_dir_all(&directory).await?;

    let mut entries = tokio::fs::read_dir(&directory).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(filename) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };

        let Some(extension) = Path::new(filename)
            .extension()
            .and_then(|v| v.to_str())
        else {
            continue;
        };

        if !AUDIO_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()) {
            continue;
        }

        // Example:
        // recording.m4a → recording.m4a.processing
        let processing_path = directory.join(format!("{filename}.processing"));

        // Atomic claim. If another worker claimed it first, try the next file.
        if tokio::fs::rename(&path, &processing_path).await.is_err() {
            continue;
        }

        return Ok(HttpResponse::Ok().json(AudioJob {
            id: filename.to_string(),
            filename: filename.to_string(),
            download_url: format!("/transcript/audio/{filename}/file"),
        }));
    }

    Ok(HttpResponse::NoContent().finish())
}

//#[get("/transcript/audio/{id}/file")]
async fn download_audio(
    path: web::Path<String>,
    data: web::Data<AppState>,
    //_user: AuthContext,
) -> Result<NamedFile, Error> {
    let id = path.into_inner();

    if !valid_id(&id) {
        return Err(actix_web::error::ErrorBadRequest("Invalid audio id"));
    }

    let file_path = transcription_dir(&data)
        .join(format!("{id}.processing"));
    log::info!("Attempting to serve audio file: {:?}", file_path);
    Ok(NamedFile::open_async(file_path).await?)
}

#[derive(Deserialize)]
struct TranscriptResult {
    transcript: String,
}

//#[post("/transcript/audio/{id}/result")]
async fn complete_audio(
    path: web::Path<String>,
    body: web::Json<TranscriptResult>,
    data: web::Data<AppState>,
    //_user: AuthContext,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    if !valid_id(&id) {
        return Err(actix_web::error::ErrorBadRequest("Invalid audio id"));
    }

    let directory = transcription_dir(&data);
    let processing_path = directory.join(format!("{id}.processing"));

    if tokio::fs::metadata(&processing_path).await.is_err() {
        return Err(actix_web::error::ErrorNotFound(
            "Processing audio file not found",
        ));
    }

    let transcript_dir = directory.join("transcripts");
    tokio::fs::create_dir_all(&transcript_dir).await?;

    let transcript_path = transcript_dir.join(format!("{id}.txt"));
    tokio::fs::write(&transcript_path, body.transcript.as_bytes()).await?;

    // Delete only after the transcript has been saved successfully.
    tokio::fs::remove_file(&processing_path).await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "completed",
        "id": id
    })))
}


pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
    .service(register(
            "upload",
            Method::POST,
            path.as_str(),
            "",
            upload,
            crate::types::MemberRole::Member,
        ))
    .service(register(
            "upload_audio",
            Method::POST,
            path.as_str(),
            "audio",
            upload_audio,
            crate::types::MemberRole::Public,
        ))
    .service(register(
            "next_audio",
            Method::GET,
            path.as_str(),
            "/transcript/audio",
            next_audio,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "download_audio",
            Method::GET,
            path.as_str(),
            "/transcript/audio/{id}/file",
            download_audio,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "complete_audio",
            Method::POST,
            path.as_str(),
            "/transcript/audio/{id}/result",
            complete_audio,
            crate::types::MemberRole::Public,
        ))
        // cleanup unused files (admin only)
        .service(register(
            "cleanup",
            Method::GET,
            path.as_str(),
            "/cleanup_unreferenced",
            cleanup_unreferenced,
            crate::types::MemberRole::Admin,
        ))
}
// .service(upload)
//     .service(cleanup_unreferenced)
