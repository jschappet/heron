use actix_web::middleware::NormalizePath;
//use actix_web::web::Payload;
use actix_web::{App, HttpResponse, HttpServer, Responder, middleware as mw, web};
//use actix_web::{App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder, get, middleware as mw, post, web};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::{ Key, SameSite};

use actix_files::{self as fs};
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy;
//mod offers;
mod types;
//mod send_email;
mod build_info;
mod services;
mod schema;
mod registration;
//mod mailing_list;
mod settings;
mod models;
mod routes;
mod errors;
mod test_support;
// mod extract_host; //rewriting to middleware
mod middleware;

//use weekly_answers::{insert_weekly_answers, get_answers_by_question, NewWeeklyAnswer};

//use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use handlebars::Handlebars;
//use registration::{create_registration, update_registration_user_id, get_registrations, NewRegistration, RegisterQuery};
mod mailmerge;
mod app_state;
use crate::db::run_migrations;
use crate::middleware::host::{HostMiddleware, HostMiddlewareService};
use crate::models::users::{self};
use crate::app_state::AppState;
use crate::services::contribute_events::ContributionDomain;
use crate::services::hosts::HostDomain;
use crate::settings::Settings;


use actix_identity::{IdentityMiddleware};

use std::sync::Mutex;
use crate::routes::{api_scope, Route, routes};

mod db;


//use actix_web_httpauth::middleware::HttpAuthentication;
use env_logger::Env;
use std::sync::Arc;
mod validator;

/* #[allow(deprecated)]
async fn not_found() -> impl Responder {

    let file = NamedFile::open("../dist/404.html").unwrap()
        .set_status_code(StatusCode::NOT_FOUND);
    
    file
} */

async fn not_found() -> impl Responder {
    let body_text = match std::fs::read_to_string("./webroot/404.html") {
        Ok(text) => text,
        Err(e) => "File Not Found".to_string(),
    };

    HttpResponse::NotFound()
        .content_type("text/html")
        .body(body_text)
}





#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenvy::dotenv().ok();



    let settings = Settings::new()
        .expect("Config failed to load");

// 2️⃣ Print route registry after scopes are built
// {
//     let api = routes::api_scope("/api");

//     let r = routes().lock().unwrap();
//     println!("Registered routes at startup:");
//     for route in r.iter() {
//         println!(
//             "{} {} -> {} (auth={:?}) [{}:{}]",
//             route.method, route.url(), route.key, route.roles, "",""
//         );
//     }
// }
    
    env_logger::Builder::from_env(Env::default().default_filter_or(settings.debug.clone())).init();
    log::info!("Starting server...");

    log::debug!("Settings: {:?}", settings.clone());
    log::info!("Build Info: {} {}", build_info::BUILD_DATE, build_info::BUILD_TIME);
    let bind_address = settings.clone().get_bind();

    let database_url = settings.database.url.clone();
    
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    {
        let mut conn = pool.get().expect("Failed to get DB connection");
        run_migrations(&mut conn).expect("Failed to run migrations");
    }

    let mut handlebars = Handlebars::new();
        handlebars
        .register_template_file("pending_registrations", "./templates/pending_registrations.hbs")
        .expect("Could not register template");


    let app_state = AppState {
        db_pool: pool.clone(),
        hb: Arc::new(handlebars),
        settings: settings.clone(),
        
    };

    let session_key = settings.web_config.cookie_key.clone();
        
    let contribution_domain = ContributionDomain::new(pool.clone());
    let host_domain = HostDomain::new(pool.clone());

    //let admin_middleware = AdminMiddleware::new();
    // {
    //     let r = routes().lock().unwrap();
    //     println!("Registered routes at startup:");
    //     for route in r.iter() {
    //     println!(
    //         "{} {} -> {} (auth={:?}) [{}:{}]",
    //         route.method, route.url(), route.key, route.roles, "",""
    //     );
    // }
    // }



    HttpServer::new(move || {

        let key = Key::from(session_key.as_bytes());
        // Build cookie separately

        let session_middleware = SessionMiddleware::builder(
            CookieSessionStore::default(),
            key,
        )
        .cookie_name(settings.web_config.cookie_name.clone()) // name of the cookie
        .cookie_secure(false)  // allow local HTTP
        .cookie_http_only(true)
        .cookie_same_site(SameSite::Lax)
        //.cookie_max_age(Duration::days(30)) // <-- 30-day session
        .build();
                


        let mut app = App::new()
            .wrap(HostMiddleware::new(app_state.db_pool.clone()))
            .app_data(web::Data::new(contribution_domain.clone())) // inject domain
            .app_data(web::Data::new(host_domain.clone())) // inject domain

            .wrap(mw::Logger::default())
            .wrap(session_middleware)
            .wrap(NormalizePath::trim())
            .wrap(IdentityMiddleware::default())
            .app_data(web::Data::new(app_state.clone()))
            //.app_data(web::Data::new(app_state.db_pool.clone()))
            
            .service(routes::api_scope("/api"))
            
            //.wrap(admin_middleware)
            //.service(routes::admin_scope())
            //.configure(routes::authentication::config)
            
            ;   
 
        // Protected routes with Basic Auth
        app = app.service(
            web::scope("/api")

            //.configure(mailing_list::config)
            .configure(registration::config)
            .configure(mailmerge::config)

        );

        // Conditionally include static files in development

        if cfg!(debug_assertions) {
        app = app.service(
            fs::Files::new("/", "./webroot/")
                .index_file("index.html")
                .default_handler(web::route().to(not_found)));

            
        }
        app
    })
    .bind(bind_address)?
    .run()
    .await
}