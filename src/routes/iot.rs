// Discovery and optional metric reports share the firmware's existing endpoint.
use crate::services::iot_metrics::{self, Reading, SaveOutcome};
use crate::{app_state::AppState, types::method::Method};
use crate::{
    middleware::host_utils::require_host,
    routes::{RoutePath, register},
};
use actix_session::SessionExt;
use actix_web::{HttpRequest, HttpResponse, Scope, web};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    net::IpAddr,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

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

fn authorize(req: &HttpRequest, data: &AppState) -> Result<(), HttpResponse> {
    let expected_token = match data.settings.web_config.iot_device_token.clone() {
        value if !value.trim().is_empty() => value,
        _ => {
            return Err(HttpResponse::ServiceUnavailable().json(json!({
                "error": "IoT registration is not configured"
            })));
        }
    };
    let supplied_token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if supplied_token != Some(expected_token.as_str()) {
        return Err(HttpResponse::Unauthorized()
            .insert_header(("WWW-Authenticate", "Bearer"))
            .json(json!({"error": "Invalid device token"})));
    }

    Ok(())
}

pub async fn post_iot(
    req: HttpRequest,
    body: web::Json<Value>,
    data: web::Data<AppState>,
) -> HttpResponse {
    if let Err(response) = authorize(&req, &data) {
        return response;
    }

    let host = match require_host(&req).await {
        Ok(host) => host,
        Err(response) => return response,
    };
    let payload = body.into_inner();
    let reading = match payload.get("reading") {
        None => None,
        Some(value) => match serde_json::from_value::<Reading>(value.clone()) {
            Ok(reading) if reading.is_valid() => Some(reading),
            _ => return HttpResponse::BadRequest().json(json!({
                "error": "Invalid reading: expected touch_count or temperature schema version 1 with report/boot IDs, timestamp, window and sensor data"
            })),
        },
    };
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
    if host_id.is_empty()
        || !host_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return HttpResponse::InternalServerError()
            .json(json!({"error": "Invalid IoT host storage key"}));
    }
    let received = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let file_name = format!(
        "{}-{}-{}.json",
        received.as_nanos(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
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
        return HttpResponse::PayloadTooLarge()
            .json(json!({"error": "Registration exceeds 16 KiB"}));
    }
    bytes.push(b'\n');
    let base = PathBuf::from(data.settings.web_config.iot_storage_dir.clone());
    let directory = base.join(&host_id);
    let saved_name = file_name.clone();
    log::info!(
        "Saving IoT registration for host {} to {}/{}",
        host_id,
        directory.display(),
        saved_name
    );
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
    })
    .await;

    match result {
        Ok(Ok(())) => {
            let mut response = json!({
                "ok": true,
                "device_id": registration.device_id,
                "registration_file": saved_name
            });
            let pool = data.db_pool.clone();
            let device_id = registration.device_id.clone();
            let numeric_host_id = host.id;
            let sensor_url = format!(
                "http://{}/sensor",
                std::net::SocketAddr::new(registration.ip_address, registration.port)
            );
            let result = web::block(move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                iot_metrics::save_device(
                    &mut conn,
                    numeric_host_id,
                    &device_id,
                    &registration.name,
                    &sensor_url,
                    received.as_secs() as i64,
                )
                .map_err(|e| e.to_string())
            })
            .await;
            if !matches!(result, Ok(Ok(_))) {
                log::error!("IoT device save failed: {:?}", result);
                return HttpResponse::InternalServerError()
                    .json(json!({"error": "Unable to save device"}));
            }
            if let Some(reading) = reading {
                let report_id = reading.report_id.clone();
                let pool = data.db_pool.clone();
                let device_id = registration.device_id.clone();
                let numeric_host_id = host.id;
                let result = web::block(move || {
                    let mut conn = pool.get().map_err(|e| e.to_string())?;
                    iot_metrics::save(
                        &mut conn,
                        numeric_host_id,
                        &device_id,
                        &reading,
                        received.as_secs() as i64,
                    )
                    .map_err(|e| e.to_string())
                })
                .await;
                match result {
                    Ok(Ok(SaveOutcome::Created)) => response["metric_duplicate"] = json!(false),
                    Ok(Ok(SaveOutcome::Duplicate)) => response["metric_duplicate"] = json!(true),
                    Ok(Ok(SaveOutcome::Conflict)) => {
                        return HttpResponse::Conflict().json(json!({
                            "error": "Report ID already exists with a different reading"
                        }));
                    }
                    error => {
                        log::error!("IoT metric save failed: {:?}", error);
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": "Unable to save metric"}));
                    }
                }
                response["report_id"] = json!(report_id);
            }
            HttpResponse::Created().json(response)
        }
        Ok(Err(error)) => {
            log::error!("IoT registration file write failed: {}", error);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Unable to save registration"}))
        }
        Err(error) => {
            log::error!("IoT registration worker failed: {}", error);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Unable to save registration"}))
        }
    }
}

// Browser viewers use the existing login session; firmware keeps its device token.
fn authorize_viewer(req: &HttpRequest, data: &AppState) -> Result<(), HttpResponse> {
    if let Ok(Some(_)) = req.get_session().get::<i32>("user_id") {
        return Ok(());
    }
    authorize(req, data)
}

pub async fn get_devices(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if let Err(response) = authorize_viewer(&req, &data) {
        return response;
    }
    let host = match require_host(&req).await {
        Ok(host) => host,
        Err(response) => return response,
    };
    let host_id = host.id;
    let pool = data.db_pool.clone();
    match web::block(move || {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        iot_metrics::devices(&mut conn, host_id).map_err(|e| e.to_string())
    })
    .await
    {
        Ok(Ok(devices)) => HttpResponse::Ok().json(json!({"devices": devices})),
        error => {
            log::error!("IoT device lookup failed: {:?}", error);
            HttpResponse::InternalServerError().json(json!({"error": "Unable to load devices"}))
        }
    }
}

#[derive(Deserialize)]
pub struct MetricsQuery {
    limit: Option<u16>,
    before_id: Option<i64>,
}

pub async fn get_metrics(
    req: HttpRequest,
    device_id: web::Path<String>,
    query: web::Query<MetricsQuery>,
    data: web::Data<AppState>,
) -> HttpResponse {
    if let Err(response) = authorize_viewer(&req, &data) {
        return response;
    }
    let host = match require_host(&req).await {
        Ok(host) => host,
        Err(response) => return response,
    };
    let limit = query.limit.unwrap_or(100);
    let before_id = query.before_id.unwrap_or(i64::MAX);
    if !valid_text(&device_id, 80) || !(1..=500).contains(&limit) || before_id <= 0 {
        return HttpResponse::BadRequest().json(
            json!({"error": "Expected valid device ID, limit 1–500 and positive before_id"}),
        );
    }
    let host_id = host.id;
    let device_id = device_id.into_inner();
    let lookup_id = device_id.clone();
    let pool = data.db_pool.clone();
    let result = web::block(move || -> Result<Vec<Value>, String> {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        iot_metrics::metrics(
            &mut conn,
            host_id,
            &lookup_id,
            before_id,
            i64::from(limit) + 1,
        )
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|row| {
            let reading: Value =
                serde_json::from_str(&row.reading_json).map_err(|e| e.to_string())?;
            Ok(json!({"id": row.id, "received_at_unix": row.received_at_unix, "reading": reading}))
        })
        .collect()
    })
    .await;
    match result {
        Ok(Ok(mut metrics)) => {
            let has_more = metrics.len() > usize::from(limit);
            metrics.truncate(usize::from(limit));
            let next_before_id = if has_more {
                metrics.last().map(|row| row["id"].clone())
            } else {
                None
            };
            HttpResponse::Ok().json(json!({"device_id": device_id, "metrics": metrics, "next_before_id": next_before_id}))
        }
        error => {
            log::error!("IoT metric lookup failed: {:?}", error);
            HttpResponse::InternalServerError().json(json!({"error": "Unable to load metrics"}))
        }
    }
}

pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register(
            "iot_sensor",
            Method::POST,
            path.as_str(),
            "",
            post_iot,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "iot_devices",
            Method::GET,
            path.as_str(),
            "/devices",
            get_devices,
            crate::types::MemberRole::Public,
        ))
        .service(register(
            "iot_device_metrics",
            Method::GET,
            path.as_str(),
            "/devices/{device_id}/metrics",
            get_metrics,
            crate::types::MemberRole::Public,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        middleware::host::{HostContext, HostInfo},
        settings::Settings,
    };
    use actix_web::{HttpMessage, http::StatusCode, test};
    use diesel::{
        connection::SimpleConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use std::{
        collections::HashMap,
        rc::Rc,
        sync::{Arc, Mutex},
    };

    #[actix_web::test]
    async fn discovery_metrics_auth_validation_and_storage_failures() {
        let directory = tempfile::tempdir().unwrap();
        let pool = Pool::builder()
            .max_size(1)
            .build(ConnectionManager::<diesel::SqliteConnection>::new(
                directory.path().join("test.db").to_str().unwrap(),
            ))
            .unwrap();
        pool.get()
            .unwrap()
            .batch_execute(
                "CREATE TABLE hosts (id INTEGER PRIMARY KEY); INSERT INTO hosts VALUES (1);",
            )
            .unwrap();
        pool.get()
            .unwrap()
            .batch_execute(include_str!(
                "../../migrations/2026-10-09-000000_iot_metrics/up.sql"
            ))
            .unwrap();
        pool.get()
            .unwrap()
            .batch_execute(include_str!(
                "../../migrations/2026-10-09-000001_iot_devices/up.sql"
            ))
            .unwrap();
        pool.get()
            .unwrap()
            .batch_execute(include_str!(
                "../../migrations/2026-10-09-000002_temperature_metrics/up.sql"
            ))
            .unwrap();
        let mut settings = Settings::default();
        settings.web_config.iot_device_token = "test-token".into();
        settings.web_config.iot_storage_dir =
            directory.path().join("records").to_str().unwrap().into();
        let data = web::Data::new(AppState {
            db_pool: pool.clone(),
            settings,
            hb: Arc::new(handlebars::Handlebars::new()),
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        });
        let request = |token: &str| {
            let req = test::TestRequest::post()
                .insert_header(("Authorization", token))
                .to_http_request();
            req.extensions_mut().insert(HostContext(Rc::new(HostInfo {
                id: 1,
                slug: "test-host".into(),
                host_name: "test".into(),
                display_name: "Test".into(),
                base_url: "".into(),
            })));
            req
        };
        let mut payload = json!({"device_id":"esp32-test", "name":"Touch counter", "ip_address":"192.168.1.10",
            "port":80,"deployed_code":{"name":"touch-counter-webserver","version":"0.2.0"},"capabilities":["touch_count"]});
        assert_eq!(
            post_iot(
                request("Bearer wrong"),
                web::Json(payload.clone()),
                data.clone()
            )
            .await
            .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            post_iot(
                request("Bearer test-token"),
                web::Json(payload.clone()),
                data.clone()
            )
            .await
            .status(),
            StatusCode::CREATED
        );
        payload["reading"] = json!({"schema":"touch_count","schema_version":1,"report_id":"boot-1","boot_id":"boot",
            "observed_at_unix":1791500000,"window_start_ms":0,"window_end_ms":300000,"window_duration_ms":300000,
            "data":{"gpio":4,"count":2,"total_count":2}});
        for duplicate in [false, true] {
            let response = post_iot(
                request("Bearer test-token"),
                web::Json(payload.clone()),
                data.clone(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::CREATED);
            let body: Value = serde_json::from_slice(
                &actix_web::body::to_bytes(response.into_body())
                    .await
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(body["metric_duplicate"], duplicate);
        }
        // A logged-in browser needs no firmware token for either read route.
        let viewer = request("");
        viewer.get_session().insert("user_id", 1_i32).unwrap();
        assert_eq!(get_devices(viewer.clone(), data.clone()).await.status(), StatusCode::OK);
        assert_eq!(get_metrics(viewer, web::Path::from("esp32-test".to_string()),
            web::Query(MetricsQuery { limit: None, before_id: None }), data.clone()).await.status(), StatusCode::OK);
        assert_eq!(get_metrics(request(""), web::Path::from("esp32-test".to_string()),
            web::Query(MetricsQuery { limit: None, before_id: None }), data.clone()).await.status(), StatusCode::UNAUTHORIZED);
        let response = get_devices(request("Bearer test-token"), data.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &actix_web::body::to_bytes(response.into_body())
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            body["devices"][0]["sensor_url"],
            "http://192.168.1.10:80/sensor"
        );
        assert_eq!(body["devices"][0]["device_id"], "esp32-test");
        assert_eq!(
            get_devices(request("Bearer wrong"), data.clone())
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let response = get_metrics(
            request("Bearer test-token"),
            web::Path::from("esp32-test".to_string()),
            web::Query(MetricsQuery {
                limit: None,
                before_id: None,
            }),
            data.clone(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &actix_web::body::to_bytes(response.into_body())
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["metrics"].as_array().unwrap().len(), 1);
        assert_eq!(body["metrics"][0]["reading"]["data"]["count"], 2);
        let other_host = request("Bearer test-token");
        other_host
            .extensions_mut()
            .insert(HostContext(Rc::new(HostInfo {
                id: 2,
                slug: "other".into(),
                host_name: "other".into(),
                display_name: "Other".into(),
                base_url: "".into(),
            })));
        let response = get_metrics(
            other_host,
            web::Path::from("esp32-test".to_string()),
            web::Query(MetricsQuery {
                limit: None,
                before_id: None,
            }),
            data.clone(),
        )
        .await;
        let body: Value = serde_json::from_slice(
            &actix_web::body::to_bytes(response.into_body())
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["metrics"], json!([]));
        assert_eq!(
            get_metrics(
                request("Bearer test-token"),
                web::Path::from("esp32-test".to_string()),
                web::Query(MetricsQuery {
                    limit: Some(0),
                    before_id: None
                }),
                data.clone()
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
        payload["reading"]["data"]["count"] = json!(3);
        assert_eq!(
            post_iot(
                request("Bearer test-token"),
                web::Json(payload.clone()),
                data.clone()
            )
            .await
            .status(),
            StatusCode::CONFLICT
        );
        payload["reading"]["schema_version"] = json!(2);
        assert_eq!(
            post_iot(
                request("Bearer test-token"),
                web::Json(payload.clone()),
                data.clone()
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
        payload["reading"]["schema_version"] = json!(1);
        pool.get()
            .unwrap()
            .batch_execute("DROP TABLE iot_metrics;")
            .unwrap();
        assert_eq!(
            post_iot(request("Bearer test-token"), web::Json(payload), data)
                .await
                .status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        let record = fs::read_dir(directory.path().join("records/test-host"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let saved: Value = serde_json::from_slice(&fs::read(record.path()).unwrap()).unwrap();
        assert_eq!(saved["host_id"], "test-host");
        assert_eq!(saved["payload"]["device_id"], "esp32-test");
    }
}
