use crate::errors::app_error::AppError;
use crate::routes::mailing_list::send_account_confirmation_email;
use crate::models::user_token::create_user_token;
use crate::registration::Registration;

use crate::schema::users::is_active;
use crate::schema::{memberships, roles, users};

use crate::schema::users::dsl::users as users_dsl;
use crate::settings::{DeployedEnvironment, Settings};
use crate::types::{JsonField, TokenPurpose};
use bcrypt::{DEFAULT_COST, verify};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

//use diesel::prelude::*;
use serde::{Deserialize, Serialize};
//use chrono::{NaiveDate, NaiveDateTime};

// User Model
#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)] // This hides the password during serialization
    pub password_hash: String,
    pub created_at: chrono::NaiveDateTime,
    pub profile_picture: Option<String>,
    pub user_details: JsonField,
    pub is_active: bool,
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    //pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: i32,
    //pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub neighborhood: Option<String>,
    //pub email: Option<String>,
    pub phone: Option<String>,
    pub show_in_directory: Option<bool>,
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        // user_details is a serde_json::Value wrapped by JsonField
        // so we extract the inner value:
        let details = &user.user_details.0;

        let show_in_dir = details
            .get("show_in_directory")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let display_name = details
            .get("display_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(String::from)
            .unwrap_or_else(|| user.username.clone());

        
        let _email_address = user.email;

        let phone_number = details
            .get("phones")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
            .map(String::from);

        log::info!("Parsed phone number: {:?}", phone_number);

        let bio = details
            .get("bio")
            .and_then(|v| v.as_str())
            .map(String::from);


        let neighborhood = details
            .get("neighborhood")
            .and_then(|v| v.as_str())
            .map(String::from);

        let image = details
            .get("image")
            .and_then(|v| v.as_str())
            .map(String::from);

        PublicUser {
            id: user.id,
            //username: user.username,
            display_name: Some(display_name),
            neighborhood,
            bio,
            image,
            //email: Some(email_address),
            phone: phone_number,
            show_in_directory: Some(show_in_dir),
        }
    }
}

pub fn get_user_by_email(conn: &mut SqliteConnection, email_val: String) -> QueryResult<User> {
    users_dsl
        .filter(users::email.eq(email_val))
        .first::<User>(conn)
}

pub fn update_user_details(
    conn: &mut SqliteConnection,
    user_id_val: i32,
    new_details: serde_json::Value,
) -> QueryResult<User> {
    diesel::update(users_dsl.find(user_id_val))
        .set(users::user_details.eq(new_details.to_string()))
        .execute(conn)?;
    users_dsl.find(user_id_val).first::<User>(conn)
}

pub fn set_password(
    conn: &mut SqliteConnection,
    user_id: i32,
    raw_password: &str,
) -> Result<User, AppError> {
    let hash = bcrypt::hash(raw_password, DEFAULT_COST)?;

    diesel::update(users_dsl.find(user_id))
        .set(users::password_hash.eq(hash))
        .execute(conn)?;
    Ok(users_dsl.find(user_id).first::<User>(conn)?)
}

pub fn find_or_create_user_by_email(
    conn: &mut SqliteConnection,
    rsvp: Registration,
) -> Result<User, AppError> {
    // Set username to phone number if email is not provided
    let new_username = if rsvp.email.is_empty() {
        rsvp.phone.clone()
    } else {
        rsvp.email.clone()
    };
    //let new_password = "$#$%NOT_SET_YET*&";

    match users_dsl
        .filter(users::email.eq(new_username.as_str()))
        .first::<User>(conn)
    {
        Ok(user) => Ok(user),
        Err(_) => {
            let user = create_user(conn, new_username.as_str(), Some(new_username.as_str()))?;
            Ok(user)
        }
    }
}

// Use bcrypt for password hashing
pub fn _create_user_legacy(
    conn: &mut SqliteConnection,
    new_username: &str,
    new_email: Option<&str>,
) -> QueryResult<User> {
    let now: NaiveDateTime = Utc::now().naive_utc();
    //let hashed_password = hash(new_password, DEFAULT_COST).expect("Failed to hash password");
    diesel::insert_into(users_dsl)
        .values((
            users::username.eq(new_username),
            users::email.eq(new_email.as_deref().unwrap_or("")),
            users::password_hash.eq(""),
            users::created_at.eq(now),
        ))
        .execute(conn)?;

    users_dsl.order(users::id.desc()).first::<User>(conn)
}

pub fn create_user(
    conn: &mut SqliteConnection,
    new_username: &str,
    new_email: Option<&str>,
) -> Result<User, AppError> {
    use crate::schema::users::dsl::*;
    use diesel::result::{DatabaseErrorKind, Error as DieselError};

    let now = Utc::now().naive_utc();

    let insert_result = diesel::insert_into(users)
        .values((
            username.eq(new_username),
            email.eq(new_email.unwrap_or("")), // ← nullable, no empty string
            password_hash.eq(""),
            created_at.eq(now),
            is_active.eq(false),
        ))
        .execute(conn);

    match insert_result {
        Ok(_) => {
            log::info!(
                "\nUser created successfully {} {}",
                new_username,
                new_email.unwrap_or("")
            );
        }

        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info)) => {
            let msg = info.message();

            let normalized = msg.chars().filter(|c| c.is_ascii()).collect::<String>();

            let column = normalized.split(':').nth(1).map(str::trim);

            let user_msg = match column {
                Some("users.email") => "Email already exists",
                Some("users.username") => "Username already exists",
                _ => "User already exists",
            };

            log::warn!("Registration conflict for {}", new_username);

            return Err(AppError::User(user_msg.to_string()));
        }

        Err(e) => {
            log::error!("Unexpected error creating user: {}", e);
            return Err(AppError::Internal("Failed to create user".to_string()));
        }
    }

    // Now it is safe to fetch
    let user = users
        .filter(username.eq(new_username))
        .first::<User>(conn)
        .map_err(|e| {
            log::error!("Failed to fetch newly created user: {}", e);
            AppError::Internal("Failed to load user".into())
        })?;

    Ok(user)
}

// Use bcrypt for password hashing
pub fn authenticate_user(
    conn: &mut SqliteConnection,
    usrname: String,
    password: String,
) -> QueryResult<User> {
    let user = users_dsl
        .filter(users::username.eq(usrname))
        .first::<User>(conn)?;
    if verify(password, &user.password_hash).expect("Failed to verify password") {
        Ok(user)
    } else {
        Err(diesel::result::Error::NotFound)
    }
}

pub fn get_users(conn: &mut SqliteConnection) -> QueryResult<Vec<User>> {
    users_dsl.load::<User>(conn)
}

pub fn get_public_users(conn: &mut SqliteConnection) -> QueryResult<Vec<User>> {
    let users_list = users::table
        .inner_join(memberships::table.on(memberships::user_id.eq(users::id)))
        .inner_join(roles::table.on(roles::id.eq(memberships::role_id)))
        .filter(roles::name.eq("member"))
        .filter(memberships::active.eq(true))
        .select(users::all_columns)
        .load::<User>(conn);
    match users_list {
        Ok(list) => {
            log::info!("Fetched {} public users", list.len());
            // Return only users with non-empty display names
            return Ok(list);
        }
        Err(e) => {
            log::error!("Error fetching public users: {:?}", e);
            return Err(e);
        }
    }
}

pub fn get_user(conn: &mut SqliteConnection, user_id_val: i32) -> QueryResult<User> {
    users_dsl.find(user_id_val).first::<User>(conn)
}

// Get user by user_id
pub fn get_user_by_id(conn: &mut SqliteConnection, user_id_val: i32) -> QueryResult<User> {
    users_dsl.find(user_id_val).first::<User>(conn)
}

pub fn get_user_by_username(conn: &mut SqliteConnection, usr_name: &str) -> QueryResult<User> {
    users_dsl
        .filter(users::username.eq(usr_name))
        .first::<User>(conn)
}

pub fn get_user_by_username_or_email(conn: &mut SqliteConnection, usr_name: &str) -> QueryResult<User> {
    users_dsl
        .filter(
            users::email.eq(usr_name)
                .or(users::username.eq(usr_name)),
        )
        .first::<User>(conn)
}

pub fn update_user(
    conn: &mut SqliteConnection,
    user_id_val: i32,
    new_password_hash: &str,
    new_email: Option<&str>,
) -> QueryResult<User> {
    diesel::update(users_dsl.find(user_id_val))
        .set((
            users::password_hash.eq(new_password_hash),
            users::email.eq(new_email.as_deref().unwrap_or("")),
        ))
        .execute(conn)?;

    users_dsl.find(user_id_val).first::<User>(conn)
}

pub fn _change_password(
    conn: &mut SqliteConnection,
    user_id_val: i32,
    new_password_hash: &str,
) -> QueryResult<User> {
    diesel::update(users_dsl.find(user_id_val))
        .set(users::password_hash.eq(new_password_hash))
        .execute(conn)?;

    users_dsl.find(user_id_val).first::<User>(conn)
}
/*
pub fn reset_password(
    conn: &mut SqliteConnection,
    //user_uuid: UUID,
    new_password_hash: &str,
    new_email: Option<&str>,
) -> QueryResult<User> {
    diesel::update(users_dsl.find(user_id_val))
        .set((
            users::password_hash.eq(new_password_hash),
            users::email.eq(new_email.as_deref().unwrap_or("")),
        ))
        .execute(conn)?;

    users_dsl.find(user_id_val).first::<User>(conn)
}

*/

pub fn delete_user(conn: &mut SqliteConnection, user_id_val: i32) -> QueryResult<usize> {
    diesel::delete(users_dsl.find(user_id_val)).execute(conn)
}

/********** CURL Examples ****************

curl -X POST http://localhost:8582/api/user/details \
    -b cookies.txt \
     -H "Content-Type: application/json" \
     -d '{
           "user_id": 14,
           "user_details": {
               "theme": "dark",
               "notifications": true,
               "bio": "Hello, I am testing!"
           }
         }'


SELECT r.name, u.id, u.username, u.email
FROM users u
JOIN memberships m ON m.user_id = u.id
JOIN roles r ON r.id = m.role_id
WHERE  m.active = 1;


********** CURL Examples ****************/

fn send_verification_or_log(
    email: &str,
    username: &str,
    token: &str,
    host_base_url: &str,
    host_display_name: &str, 
    settings: &Settings,
) -> Result<(), AppError> {
    match send_account_confirmation_email(email, username, token, host_base_url, host_display_name, settings) {
        Ok(_) => Ok(()),
        Err(e) if settings.environment == DeployedEnvironment::Development => {
            log::warn!("DEV: email skipped: {}", e);
            log::info!("DEV verify token: {}", token);
            Ok(())
        }
        Err(e) => {
            log::error!("Email send failed: {}", e);
            Err(AppError::Internal(
                "Failed to send verification email".into(),
            ))
        }
    }
}

pub fn activate_user(conn: &mut SqliteConnection, user_id: i32) -> Result<(), AppError> {
    diesel::update(users_dsl.find(user_id))
        .set(is_active.eq(true))
        .execute(conn)?;

    Ok(())
}

pub fn register_user(
    conn: &mut SqliteConnection,
    username: &str,
    email: &str,
    host_base_url: &str,
    host_display_name: &str,
    settings: &Settings,
) -> Result<User, AppError> {
    conn.transaction(|conn| {
        // 1. Create user
        let user = create_user(conn, username, Some(email))?;

        // 2. Create verification token
        let token = create_user_token(conn, user.id, TokenPurpose::VerifyAccount, 60)?;

        let _ = send_verification_or_log(email, username, &token, host_base_url, host_display_name,  &settings)?;

        Ok(user)
    })
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{models::user_token::verify_user_token, test_support::db::setup_test_db};

    #[test]
    fn send_verification_or_log_works() {
            dotenvy::dotenv().ok();


    let settings = Settings::new()
        .expect("Config failed to load");

        let result = send_verification_or_log(
            "jschappet@gmail.com", "testuser", "SOMETOKEN123", "http://localhost/", "TEST", &settings,
        );
        assert!(result.is_ok());
    }


    #[test]
    fn create_user_works() {
        let (_tmp_dir, pool, _this_user_id) = setup_test_db();
        let mut conn = pool.get().unwrap();

        let new_user = NewUser {
            username: "testuser_1".to_string(),
            email: "mail@example.com".to_string(),
        };
        let result = create_user(&mut conn, &new_user.username, Some(&new_user.email)).unwrap();
        log::info!("User created: {:?}", result);
        assert_eq!(result.username, "testuser_1");

        let result = create_user(&mut conn, &new_user.username, Some("email@to.com"));
        assert!(result.is_err());
        match result {
            Err(AppError::User(msg)) => assert_eq!(msg, "Username already exists"),
            Err(e) => panic!("Unexpected DB error: {:?}", e),

            Ok(_) => panic!("Expected an error, but got Ok"),
        }

        let result = create_user(&mut conn, "same_email_1", Some("testuser_2@example.com"));
        assert!(!result.is_err());

        let result = create_user(&mut conn, "same_email_2", Some("testuser_2@example.com"));
        assert!(result.is_err());
        match result {
            Err(AppError::User(msg)) => assert_eq!(msg, "Email already exists"),
            Err(e) => panic!("Unexpected DB error: {:?}", e),
            Ok(_) => panic!("Expected an error, but got Ok"),
        }
    }

    #[test]
    fn test_verify_user_token() {
        let (_tmp_dir, pool, _this_user_id) = setup_test_db();
        let mut conn = pool.get().unwrap();

        let new_user = NewUser {
            username: "testuser_1".to_string(),
            email: "<EMAIL>".to_string(),
        };
        let user = create_user(&mut conn, &new_user.username, Some(&new_user.email)).unwrap();
        let token = create_user_token(&mut conn, user.id, TokenPurpose::VerifyAccount, 60).unwrap();
        let verified = verify_user_token(&mut conn, &token, TokenPurpose::VerifyAccount).unwrap();
        assert_eq!(verified, user.id);

        let verified = verify_user_token(&mut conn, "BAD_TOKEN", TokenPurpose::VerifyAccount);
        assert!(verified.is_err());
    }
}
