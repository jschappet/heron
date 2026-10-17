use actix_web::{HttpResponse, Scope, web};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{app_state::AppState, errors::app_error::AppError, middleware::host::HostContext,
    models::memberships::Membership, routes::{register, RoutePath}, schema::{memberships, roles, users},
    types::{MemberRole, method::Method}, validator::{AuthContext, require_role_for_host}};

#[derive(Deserialize)]
pub struct AssignRole { pub user_id: i32, pub role_id: i32 }

#[derive(Serialize)]
pub struct Assignment {
    pub id: i32, pub user_id: i32, pub role_id: i32, pub host_id: i32, pub active: bool,
}
impl From<Membership> for Assignment {
    fn from(m: Membership) -> Self {
        Self { id: m.id, user_id: m.user_id, role_id: m.role_id, host_id: m.host_id, active: m.active }
    }
}

#[derive(Serialize)]
pub struct SiteUser { pub id: i32, pub username: String, pub email: String }

#[derive(Deserialize)]
pub struct UserLookup { pub email: Option<String> }

pub async fn site_users(query: web::Query<UserLookup>, data: web::Data<AppState>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    require_role_for_host(&auth, host.0.id, &[MemberRole::Admin])?;
    let mut conn = data.db_conn()?;
    // Normal lists stay site-scoped. An exact email lookup allows an administrator
    // to assign an existing account its first role here without exposing other-site memberships.
    let mut query_users = users::table.filter(users::is_active.eq(true)).into_boxed();
    if let Some(email) = query.email.as_ref().filter(|s| !s.trim().is_empty()) {
        query_users = query_users.filter(users::email.eq(email.trim()));
    } else {
        query_users = query_users.filter(users::id.eq_any(memberships::table.filter(memberships::host_id.eq(host.0.id)).select(memberships::user_id)));
    }
    let rows = query_users.select((users::id, users::username, users::email))
        .order(users::username.asc()).load::<(i32, String, String)>(&mut conn)?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(id, username, email)| SiteUser { id, username, email }).collect::<Vec<_>>()))
}

fn assign(conn: &mut SqliteConnection, host_id: i32, request: &AssignRole) -> Result<Assignment, AppError> {
    conn.immediate_transaction(|conn| {
        users::table.find(request.user_id).filter(users::is_active.eq(true)).select(users::id).first::<i32>(conn)?;
        let role = roles::table.find(request.role_id).select(roles::name).first::<String>(conn)?;
        if role == "public" || !MemberRole::all().iter().any(|r| r.value == role) {
            return Err(AppError::BadRequest("Unsupported role".into()));
        }
        let existing = memberships::table.filter(memberships::host_id.eq(host_id))
            .filter(memberships::user_id.eq(request.user_id)).filter(memberships::role_id.eq(request.role_id))
            .order(memberships::active.desc()).first::<Membership>(conn).optional()?;
        let id = if let Some(m) = existing {
            diesel::update(memberships::table.find(m.id)).set((memberships::active.eq(true), memberships::ended_at.eq(None::<chrono::NaiveDateTime>))).execute(conn)?;
            m.id
        } else {
            diesel::insert_into(memberships::table).values((memberships::host_id.eq(host_id), memberships::user_id.eq(request.user_id), memberships::role_id.eq(request.role_id), memberships::active.eq(true))).execute(conn)?;
            memberships::table.order(memberships::id.desc()).select(memberships::id).first(conn)?
        };
        Ok(memberships::table.find(id).first::<Membership>(conn)?.into())
    })
}

fn revoke(conn: &mut SqliteConnection, host_id: i32, id: i32) -> Result<(), AppError> {
    conn.immediate_transaction(|conn| {
        let m = memberships::table.find(id).filter(memberships::host_id.eq(host_id)).first::<Membership>(conn)?;
        let role = roles::table.find(m.role_id).select(roles::name).first::<String>(conn)?;
        if m.active && role == "admin" {
            let remaining: i64 = memberships::table.inner_join(roles::table).inner_join(users::table)
                .filter(memberships::host_id.eq(host_id)).filter(memberships::id.ne(id))
                .filter(memberships::active.eq(true)).filter(users::is_active.eq(true))
                .filter(roles::name.eq("admin")).count().get_result(conn)?;
            if remaining == 0 { return Err(AppError::BadRequest("The site's last active administrator cannot be removed".into())); }
        }
        diesel::update(memberships::table.find(id).filter(memberships::host_id.eq(host_id)))
            .set((memberships::active.eq(false), memberships::ended_at.eq(Some(chrono::Utc::now().naive_utc())))).execute(conn)?;
        Ok(())
    })
}

pub async fn create_membership_api(data: web::Data<AppState>, auth: AuthContext, host: HostContext, new_m: web::Json<AssignRole>) -> Result<HttpResponse, AppError> {
    require_role_for_host(&auth, host.0.id, &[MemberRole::Admin])?;
    Ok(HttpResponse::Ok().json(assign(&mut *data.db_conn()?, host.0.id, &new_m)?))
}

pub async fn get_memberships_api(data: web::Data<AppState>, auth: AuthContext, host: HostContext) -> Result<HttpResponse, AppError> {
    require_role_for_host(&auth, host.0.id, &[MemberRole::Admin])?;
    let list = memberships::table.filter(memberships::host_id.eq(host.0.id)).filter(memberships::active.eq(true)).load::<Membership>(&mut data.db_conn()?)?;
    Ok(HttpResponse::Ok().json(list.into_iter().map(Assignment::from).collect::<Vec<_>>()))
}

pub async fn get_memberships_for_user_api(data: web::Data<AppState>, auth: AuthContext, host: HostContext, id: web::Path<i32>) -> Result<HttpResponse, AppError> {
    require_role_for_host(&auth, host.0.id, &[MemberRole::Admin])?;
    let list = memberships::table.filter(memberships::host_id.eq(host.0.id)).filter(memberships::user_id.eq(*id)).filter(memberships::active.eq(true)).load::<Membership>(&mut data.db_conn()?)?;
    Ok(HttpResponse::Ok().json(list.into_iter().map(Assignment::from).collect::<Vec<_>>()))
}

pub async fn delete_membership_api(data: web::Data<AppState>, auth: AuthContext, host: HostContext, id: web::Path<i32>) -> Result<HttpResponse, AppError> {
    require_role_for_host(&auth, host.0.id, &[MemberRole::Admin])?;
    revoke(&mut *data.db_conn()?, host.0.id, *id)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"revoked": *id})))
}

pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register("site_users", Method::GET, path.as_str(), "users", site_users, MemberRole::Admin))
        .service(register("get_memberships", Method::GET, path.as_str(), "", get_memberships_api, MemberRole::Admin))
        .service(register("assign_site_role", Method::POST, path.as_str(), "", create_membership_api, MemberRole::Admin))
        .service(register("revoke_site_role", Method::DELETE, path.as_str(), "{id}", delete_membership_api, MemberRole::Admin))
}

pub fn admin_scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register("create_membership", Method::POST, path.as_str(), "", create_membership_api, MemberRole::Admin))
        .service(register("get_memberships_for_user", Method::GET, path.as_str(), "user/{id}", get_memberships_for_user_api, MemberRole::Admin))
        .service(register("delete_membership", Method::DELETE, path.as_str(), "{id}", delete_membership_api, MemberRole::Admin))
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;
    fn database() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        conn.batch_execute("CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT, email TEXT, is_active BOOLEAN NOT NULL);
            CREATE TABLE roles (id INTEGER PRIMARY KEY, name TEXT);
            CREATE TABLE memberships (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, role_id INTEGER NOT NULL, host_id INTEGER NOT NULL,
                active BOOLEAN NOT NULL DEFAULT 1, created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, ended_at TIMESTAMP);
            CREATE UNIQUE INDEX membership_unique ON memberships(user_id,host_id,role_id,active);
            INSERT INTO users VALUES (1,'Admin','admin@test',1),(2,'Member','member@test',1),(3,'Other site','other@test',1);
            INSERT INTO roles VALUES (1,'admin'),(2,'member'),(3,'reviewer'),(4,'public');
            INSERT INTO memberships (id,user_id,role_id,host_id) VALUES (1,1,1,1),(2,2,2,1),(3,3,1,2);").unwrap();
        conn
    }
    #[test]
    fn assignments_are_site_scoped_idempotent_and_reversible() {
        let mut conn = database();
        let added = assign(&mut conn, 1, &AssignRole {user_id: 3, role_id: 2}).unwrap();
        assert_eq!(added.host_id, 1);
        assert!(memberships::table.find(3).first::<Membership>(&mut conn).unwrap().active);
        assert!(assign(&mut conn, 1, &AssignRole {user_id: 99, role_id: 2}).is_err());
        assert!(assign(&mut conn, 1, &AssignRole {user_id: 2, role_id: 4}).is_err());
        let request = AssignRole {user_id: 2, role_id: 3};
        let first = assign(&mut conn, 1, &request).unwrap();
        assert_eq!(first.id, assign(&mut conn, 1, &request).unwrap().id);
        assert!(revoke(&mut conn, 2, first.id).is_err());
        revoke(&mut conn, 1, first.id).unwrap();
        assert_eq!(assign(&mut conn, 1, &request).unwrap().id, first.id);
        let spoofed: AssignRole = serde_json::from_value(serde_json::json!({"user_id":2,"role_id":3,"host_id":2})).unwrap();
        assert_eq!(assign(&mut conn, 1, &spoofed).unwrap().host_id, 1);
    }
    #[test]
    fn last_admin_is_retained_and_other_site_admin_does_not_count() {
        let mut conn = database();
        assert!(revoke(&mut conn, 1, 1).is_err());
        assign(&mut conn, 1, &AssignRole {user_id: 2, role_id: 1}).unwrap();
        revoke(&mut conn, 1, 1).unwrap();
        assert!(!memberships::table.find(1).first::<Membership>(&mut conn).unwrap().active);
    }
}
