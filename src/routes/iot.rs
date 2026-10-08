// Add this as a backend module and register post_iot in your existing scope.
// Requires the existing actix_web, serde, serde_json and log dependencies.
use actix_web::{HttpRequest, HttpResponse, Scope, web};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::{app_state::AppState, types::method::Method};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    net::IpAddr,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use crate::{middleware::host_utils::require_host, routes::{RoutePath, register}};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
struct Firmware {
    name: String,
    version: String,
}

#[derive(Deserialize)]
struct Registration {
    device_id: String,
    name: String,
    ip_address: IpAddr,
    port: u16,
    deployed_code: Firmware,
    capabilities: Vec<String>,
}

fn valid_text(text: &str, maximum: usize) -> bool {
    !text.trim().is_empty() && text.len() <= maximum && !text.chars().any(char::is_control)
}

pub async fn post_iot(
    req: HttpRequest, 
    body: web::Json<Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    // Set IOT_DEVICE_TOKEN in the backend process/container environment.
    // This initial version uses one shared token; it is NOT user login auth.

        //let expected_token = data.settings.web_config.iot_device_token.clone();

    let expected_token = match data.settings.web_config.iot_device_token.clone() {
        value if !value.trim().is_empty() => value,
        _ => return HttpResponse::ServiceUnavailable().json(json!({
            "error": "IoT registration is not configured"
        })),
    };
    let supplied_token = req.headers().get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if supplied_token != Some(expected_token.as_str()) {
        return HttpResponse::Unauthorized()
            .insert_header(("WWW-Authenticate", "Bearer"))
            .json(json!({"error": "Invalid device token"}));
    }

    let host = match require_host(&req).await {
        Ok(host) => host,
        Err(response) => return response,
    };
    let payload = body.into_inner();
    let registration: Registration = match serde_json::from_value(payload.clone()) {
        Ok(value) => value,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "error": "Expected device_id, name, ip_address, port, deployed_code {name, version}, and capabilities"
        })),
    };
    if !valid_text(&registration.device_id, 80)
        || !valid_text(&registration.name, 160)
        || !valid_text(&registration.deployed_code.name, 120)
        || !valid_text(&registration.deployed_code.version, 80)
        || registration.port == 0
        || registration.capabilities.len() > 32
        || registration.capabilities.iter().any(|c| !valid_text(c, 80))
    {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid registration fields"}));
    }
    // Parse the IP for validation; no attempt is made to contact that IP.
    let _ = registration.ip_address;

    // Directory name comes from trusted HostContext, never the device payload.
    let host_id = host.slug.clone();
    //if host_id.is_empty() || !host_id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
    //    log::error!("Unexpected IoT host ID format");
    //    return HttpResponse::InternalServerError().json(json!({"error": "Unable to save registration"}));
    //}
    let received = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let file_name = format!("{}-{}-{}.json", received.as_nanos(), std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed));
    let record = json!({
        "host_id": host_id,
        "received_at_unix": received.as_secs(),
        "payload": payload
    });
    let mut bytes = match serde_json::to_vec_pretty(&record) {
        Ok(value) => value,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if bytes.len() > 16 * 1024 {
        return HttpResponse::PayloadTooLarge().json(json!({"error": "Registration exceeds 16 KiB"}));
    }
    bytes.push(b'\n');
    let base = PathBuf::from(data.settings.web_config.iot_storage_dir.clone());
    let directory = base.join(&host_id);
    let saved_name = file_name.clone();
    log::info!("Saving IoT registration for host {} to {}/{}", host_id, directory.display(), saved_name);
    // Blocking disk operations run outside Actix's async request workers.
    let result = web::block(move || -> io::Result<()> {
        fs::create_dir_all(&directory)?;
        let path = directory.join(file_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&path)?;
        if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        Ok(())
    }).await;

    match result {
        Ok(Ok(())) => HttpResponse::Created().json(json!({
            "ok": true,
            "device_id": registration.device_id,
            "registration_file": saved_name
        })),
        Ok(Err(error)) => {
            log::error!("IoT registration file write failed: {}", error);
            HttpResponse::InternalServerError().json(json!({"error": "Unable to save registration"}))
        }
        Err(error) => {
            log::error!("IoT registration worker failed: {}", error);
            HttpResponse::InternalServerError().json(json!({"error": "Unable to save registration"}))
        }
    }
}


pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register("iot_sensor", Method::POST, path.as_str(), "", post_iot, crate::types::MemberRole::Public))
}
