use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable, SqliteConnection};



#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::user_tokens)]
pub struct UserToken {
    pub id: i32,
    pub user_id: i32,
    pub token_hash: String,
    pub purpose: TokenPurpose,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_tokens)]
pub struct NewUserToken<'a> {
    pub user_id: i32,
    pub token_hash: &'a str,
    pub purpose: TokenPurpose,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}


use uuid::Uuid;
use sha2::{Digest, Sha256};

use crate::errors::app_error::AppError;
use crate::errors::auth_error::AuthError;

use crate::types::TokenPurpose;
use diesel::prelude::*;

pub fn create_user_token(
    conn: &mut SqliteConnection,
    in_user_id: i32,
    purpose_val: TokenPurpose,
    ttl_minutes: i64,
) -> Result<String, AppError> {
    use crate::schema::user_tokens::dsl::*;
    
    let now = Utc::now().naive_utc();
    let expires = now + chrono::Duration::minutes(ttl_minutes);

    // 1. Generate raw token
    let raw_token = Uuid::new_v4().to_string();

    // 2. Hash it (tokens are passwords)
    let token_hash_str = {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    log::debug!("Hashed token: {}", token_hash_str);

    // 3. Invalidate existing unused tokens of same purpose
    diesel::update(
        user_tokens
            .filter(user_id.eq(in_user_id))
            .filter(purpose.eq(purpose_val))
            .filter(used_at.is_null()),
    )
    .set(used_at.eq(now))
    .execute(conn)
    .map_err(|e| {
        log::error!("Failed to invalidate old tokens: {}", e);
        AppError::Internal("Failed to rotate token".into())
    })?;

    // 4. Insert new token
    let new_token = NewUserToken {
        user_id: in_user_id,
        token_hash: &token_hash_str,
        purpose: purpose_val,
        expires_at: expires,
        created_at: now,
    };

    diesel::insert_into(user_tokens)
        .values(new_token)
        .execute(conn)
        .map_err(|e| {
            log::error!("Failed to insert new token: {}", e);
            AppError::Internal("Failed to create token".into())
        })?;

    // 5. DEV ergonomics (safe because token is already issued)
    log::info!(
        "Created {:?} token for user {}",
        purpose_val,
        in_user_id
    );

    Ok(raw_token)
}

pub fn verify_user_token(
    conn: &mut SqliteConnection,
    raw_token: &str,
    purpose_val: TokenPurpose,
) -> Result<i32, AppError> {
    use crate::schema::user_tokens::dsl::*;

    let now = Utc::now().naive_utc();

    // 1. Hash incoming token
    let token_hash_str = {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    log::debug!("Hashed token: {}", token_hash_str);
    // 2. Find matching unused token
    let token = user_tokens
        .filter(token_hash.eq(&token_hash_str))
        .filter(purpose.eq(purpose_val))
        .filter(used_at.is_null())
        .first::<UserToken>(conn)
        .optional()
        .map_err(|e| {
            log::error!("Token lookup failed: {}", e);
            AppError::Internal("Token verification failed".into())
        })?
        .ok_or(AppError::Auth(AuthError::InvalidToken("Invalid or expired token".into())))?;

    // 3. Expiration check
    if token.expires_at < now {
        // Burn it anyway — expired tokens shouldn't linger
        diesel::update(user_tokens.find(token.id))
            .set(used_at.eq(now))
            .execute(conn)
            .ok();

        return Err(AppError::Auth(AuthError::InvalidToken("Invalid or expired token".into())));
    }

    // 4. Mark token as used
    diesel::update(user_tokens.find(token.id))
        .set(used_at.eq(now))
        .execute(conn)
        .map_err(|e| {
            log::error!("Failed to mark token as used: {}", e);
            AppError::Internal("Token verification failed".into())
        })?;


    // 5. Return owning user
    Ok(token.user_id)
}


#[test]
fn create_user_token_logic_works() {
    use crate::schema::user_tokens::dsl::*;
use crate::test_support::db::setup_test_db;
use crate::schema::user_tokens::{ purpose, user_id};

    let (_tmp_dir, pool, _this_user_id)
        = setup_test_db();
    let mut conn = pool.get().unwrap();
use crate::models::users::create_user as crt_user;

    let user = crt_user(
        &mut conn,
        "testuser_1",
        Some("testuser@example.com"),
    ).unwrap();

    let raw_token = create_user_token(
        &mut conn,
        user.id,
        TokenPurpose::VerifyAccount,
        60,
    )
    .unwrap();
        log::info!("Token created for user {}", user.id);

    // Fetch token from DB
    let token = user_tokens
        .filter(user_id.eq(user.id))
        .filter(purpose.eq(TokenPurpose::VerifyAccount))
        .first::<UserToken>(&mut conn)
        .unwrap();

    // Raw token should not be stored
    assert_ne!(raw_token, token.token_hash);

    // Token should be unused
    assert!(token.used_at.is_none());

    // Token should expire in the future
    assert!(token.expires_at > chrono::Utc::now().naive_utc());
}

#[test]
fn creating_new_token_invalidates_previous_token_of_same_purpose() {
    use crate::test_support::{db::setup_test_db,  init_test_logger};
use crate::schema::user_tokens::{created_at, dsl, purpose, user_id};

    init_test_logger();

    let (_tmp_dir, pool, this_user_id)
        = setup_test_db();
    let mut conn = pool.get().unwrap();

    // Create first token
    create_user_token(
        &mut conn,
        this_user_id,
        TokenPurpose::ResetPassword,
        60,
    )
    .unwrap();

    let first = dsl::user_tokens
        .filter(user_id.eq(this_user_id))
        .filter(purpose.eq(TokenPurpose::ResetPassword))
        .first::<UserToken>(&mut conn)
        .unwrap();

    assert!(first.used_at.is_none());

    // Create second token of same purpose
    create_user_token(
        &mut conn,
        this_user_id,
        TokenPurpose::ResetPassword,
        60,
    )
    .unwrap();

    let tokens = dsl::user_tokens
        .filter(user_id.eq(this_user_id))
        .filter(purpose.eq(TokenPurpose::ResetPassword))
        .order(created_at.asc())
        .load::<UserToken>(&mut conn)
        .unwrap();

    assert_eq!(tokens.len(), 2);

    assert!(
        tokens[0].used_at.is_some(),
        "old token should be invalidated"
    );

    assert!(
        tokens[1].used_at.is_none(),
        "new token should be active"
    );
}


#[test]
fn token_cannot_be_used_after_manual_invalidation() {
    use crate::schema::user_tokens::{ dsl, used_at, user_id};

    use crate::test_support::{db::setup_test_db,  init_test_logger};

    init_test_logger();
    
    use diesel::result::Error as DieselError;

    let (_tmp_dir, pool, this_user_id)
        = setup_test_db();
    let mut conn = pool.get().unwrap();

    let _raw = create_user_token(
        &mut conn,
        this_user_id,
        TokenPurpose::VerifyAccount,
        60,
    )
    .unwrap();

    // Manually invalidate
    diesel::update(dsl::user_tokens)
        .set(used_at.eq(chrono::Utc::now().naive_utc()))
        .execute(&mut conn)
        .unwrap();

    match dsl::user_tokens
        .filter(user_id.eq(this_user_id))
        .filter(used_at.is_null())
        .first::<UserToken>(&mut conn)
    {
        Ok(_) => panic!("Token should not be usable"),
        Err(DieselError::NotFound) => {
            // ✅ expected
        }
        Err(e) => panic!("Unexpected DB error: {:?}", e),
    }
}
