//! Development-only virtual hosts. The same mapping selects files and API identity.
use actix_files::Files;
use actix_web::{HttpResponse, guard, http::header::HeaderMap, web};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct DevSite {
    pub hostname: String,
    pub root: PathBuf,
    /// Exact host_name of the existing database identity (not its local alias).
    pub api_host: String,
}

pub fn request_host(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("X-Forwarded-Host")
        .or_else(|| headers.get("Host"))
        .and_then(|value| value.to_str().ok())
}

pub fn hostname(authority: &str) -> String {
    authority
        .trim()
        .split(':')
        .next()
        .unwrap_or("")
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

pub fn find_site<'a>(sites: &'a [DevSite], authority: &str) -> Option<&'a DevSite> {
    let name = hostname(authority);
    sites.iter().find(|site| hostname(&site.hostname) == name)
}

pub fn static_files(root: PathBuf, host: Option<String>) -> Files {
    let error_page = root.join("404.html");
    Files::new("/", root)
        .guard(guard::fn_guard(move |ctx| {
            let path = ctx.head().uri.path();
            // Missing API routes must never fall through to static files.
            path != "/api"
                && !path.starts_with("/api/")
                && host.as_ref().is_none_or(|expected| {
                    request_host(&ctx.head().headers)
                        .is_some_and(|actual| hostname(actual) == hostname(expected))
                })
        }))
        .index_file("index.html")
        .default_handler(web::route().to(move || {
            let error_page = error_page.clone();
            async move {
                let body = tokio::fs::read_to_string(error_page)
                    .await
                    .unwrap_or_else(|_| "File Not Found".to_string());
                HttpResponse::NotFound()
                    .content_type("text/html")
                    .body(body)
            }
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::host::{HostContext, HostMiddleware};
    use actix_web::{App, test};
    use diesel::{
        RunQueryDsl, SqliteConnection,
        r2d2::{ConnectionManager, Pool},
    };

    #[actix_web::test]
    async fn hostnames_select_both_files_and_api_identity() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ConnectionManager::<SqliteConnection>::new(
            temp.path().join("hosts.db").to_string_lossy().to_string(),
        );
        let pool = Pool::builder().max_size(1).build(manager).unwrap();
        diesel::sql_query("CREATE TABLE hosts (id INTEGER NOT NULL, slug TEXT NOT NULL, host_name TEXT NOT NULL, display_name TEXT NOT NULL, base_url TEXT NOT NULL, created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, active BOOLEAN NOT NULL)")
            .execute(&mut pool.get().unwrap()).unwrap();
        let mut sites = Vec::new();
        for (id, name) in [(1, "regen"), (2, "revillage"), (3, "workshop")] {
            let root = temp.path().join(name);
            std::fs::create_dir(&root).unwrap();
            std::fs::write(root.join("index.html"), name).unwrap();
            std::fs::write(root.join("404.html"), format!("{name} missing")).unwrap();
            std::fs::create_dir(root.join("api")).unwrap();
            std::fs::write(root.join("api/leak.txt"), "must not serve").unwrap();
            diesel::sql_query(format!("INSERT INTO hosts (id,slug,host_name,display_name,base_url,active) VALUES ({id},'{name}','{name}.example','{name}','https://{name}.example',1)"))
                .execute(&mut pool.get().unwrap()).unwrap();
            sites.push(DevSite {
                hostname: format!("{name}.localhost"),
                root,
                api_host: format!("{name}.example"),
            });
        }
        let configured_sites = sites.clone();
        let app = test::init_service(
            App::new()
                .wrap(HostMiddleware::new(pool.clone(), sites))
                .route(
                    "/api/identity",
                    web::get().to(|host: HostContext| async move {
                        HttpResponse::Ok().body(host.0.slug.clone())
                    }),
                )
                .configure(move |cfg| {
                    for site in configured_sites {
                        cfg.service(static_files(site.root, Some(site.hostname)));
                    }
                }),
        )
        .await;

        for name in ["regen", "revillage", "workshop"] {
            for path in ["/", "/api/identity"] {
                let req = test::TestRequest::get()
                    .uri(path)
                    .insert_header(("Host", format!("{}.LOCALHOST:8582", name.to_uppercase())))
                    .to_request();
                let response = test::call_service(&app, req).await;
                assert_eq!(response.status(), 200);
                assert_eq!(test::read_body(response).await, name);
            }
            let req = test::TestRequest::get()
                .uri("/missing")
                .insert_header(("Host", format!("{name}.localhost:8582")))
                .to_request();
            let response = test::call_service(&app, req).await;
            assert_eq!(response.status(), 404);
            assert_eq!(test::read_body(response).await, format!("{name} missing"));
        }
        let req = test::TestRequest::get()
            .uri("/api/leak.txt")
            .insert_header(("Host", "regen.localhost:8582"))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 404);
        // Proxy host selection must agree between static routing and API middleware.
        for path in ["/", "/api/identity"] {
            let req = test::TestRequest::get()
                .uri(path)
                .insert_header(("Host", "localhost:8582"))
                .insert_header(("X-Forwarded-Host", "workshop.localhost:8582"))
                .to_request();
            assert_eq!(test::call_and_read_body(&app, req).await, "workshop");
        }
        diesel::sql_query("DELETE FROM hosts WHERE slug = 'workshop'")
            .execute(&mut pool.get().unwrap())
            .unwrap();
        let req = test::TestRequest::get()
            .uri("/api/identity")
            .insert_header(("Host", "workshop.localhost:8582"))
            .to_request();
        let error = test::try_call_service(&app, req).await.unwrap_err();
        assert_eq!(error.as_response_error().status_code(), 503);
    }

    #[actix_web::test]
    async fn legacy_webroot_and_plain_404_still_work() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("index.html"), "legacy").unwrap();
        let app =
            test::init_service(App::new().service(static_files(temp.path().into(), None))).await;
        assert_eq!(
            test::call_and_read_body(&app, test::TestRequest::get().uri("/").to_request()).await,
            "legacy"
        );
        let response =
            test::call_service(&app, test::TestRequest::get().uri("/missing").to_request()).await;
        assert_eq!(response.status(), 404);
        assert_eq!(test::read_body(response).await, "File Not Found");
    }

    #[actix_web::test]
    async fn production_does_not_enable_local_aliases() {
        let mut settings = crate::settings::Settings::default();
        settings.dev_sites.push(DevSite {
            hostname: "regen.localhost".into(),
            root: "unused".into(),
            api_host: "regen.example".into(),
        });
        assert_eq!(
            settings.active_dev_sites().len(),
            usize::from(cfg!(debug_assertions))
        );
        settings.environment = crate::settings::DeployedEnvironment::Production;
        assert!(settings.active_dev_sites().is_empty());
        settings.environment = crate::settings::DeployedEnvironment::Staging;
        assert!(settings.active_dev_sites().is_empty());
    }
}
