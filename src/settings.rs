use std::env;

use config::{Config, ConfigError, Environment, File};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(unused)]
pub struct Database {
    pub url: String,
    pub migration: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(unused)]
pub struct Gpt {
    pub api_key: String, 
    pub url: String,
    pub max_tokens: u32,
    pub model: String,
    pub temperature: f32,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(unused)]
pub struct WebConfig {
    pub host: String,
    pub port: String,
    pub context_path: String,
    pub cookie_key: String,
    pub cookie_name: String,
    pub login_url: String, 
    pub upload_dir: String,
    pub image_site_path: String,
}


#[derive(Debug, Deserialize, Clone, Default)]
#[allow(unused)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(unused)]
pub struct Email {
    pub smtp_from_email: String,
    pub verify_token_subject: String,
    pub verify_token_text: String,
    pub base_url: String,
    pub reset_password_subject: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(unused)]
pub struct Twilio {
    pub account_sid: String,
    pub auth_token: String,
    pub phone_number: String, 
}


#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq, Hash)]

pub enum DeployedEnvironment {
    #[serde(rename = "development")]
    #[default]
    Development,
    Staging,
    Production,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(unused)]
pub struct Settings {
    pub environment: DeployedEnvironment,

    pub debug: String,
    pub database: Database,
    pub web_config: WebConfig,
    pub twilio: Twilio,
    pub email: Email,
    pub smtp: SmtpConfig,
    pub gpt: Gpt,
    pub templates: String, 
}

impl Settings {
    
    pub fn get_bind(self) -> String {
        format!("{}:{}", self.web_config.host, self.web_config.port)
    } 

    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/application"))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(
                File::with_name(&format!("config/{run_mode}"))
                    .required(false),
            )
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(Environment::with_prefix("APP_"))
            // You may also programmatically change settings
            //.set_override("database.url", "postgres://")?
            .build()?;

        // Now that we're done, let's access our configuration
        //println!("debug: {:?}", s.get_bool("debug"));
        //println!("database: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}