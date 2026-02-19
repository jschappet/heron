use crate::errors::app_error::AppError;
use crate::errors::auth_error::AuthError;
use crate::middleware::host_utils::require_host_id;
//use crate::types::{AdminContext, MemberRole, MembershipContext};
use crate::{app_state::AppState, models::users::get_user};
use actix_web::HttpMessage;

use actix_session::Session;
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use diesel::prelude::*;
use futures::future::{Ready, ready};

use diesel::Queryable;

use crate::types::MemberRole;

#[derive(Queryable, Clone, Debug)]
pub struct MembershipContext {
    pub host_id: i32,     // The host this role applies to
    pub role: MemberRole, // e.g., Admin, Reviewer
}

#[derive(Clone)]
pub struct AuthContext {
    pub user_id: i32,
    //pub user: User,
    //pub roles: Vec<crate::types::MemberRole>, // e.g. ["admin", "member"]
    pub memberships: Vec<MembershipContext>, // filtered to hosts where the user has admin privileges
}

impl AuthContext {
    /// Return a list of roles the user has across hosts
    pub fn get_roles(&self) -> Vec<MemberRole> {
        self.memberships.iter().map(|m| m.role.clone()).collect()
    }

    pub fn is_admin(&self) -> bool {
        // 3️⃣ Check admin membership
        self.memberships
            .iter()
            .any(|m| matches!(m.role, MemberRole::Admin))
          // And not guest
            
    }

    pub fn is_reviewer(&self) -> bool {
        // 3️⃣ Check admin membership
        self.memberships
            .iter()
            .any(|m| matches!(m.role, MemberRole::Reviewer))
    }
}

impl FromRequest for AuthContext {
    type Error = AuthError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let session = match Session::from_request(req, payload).into_inner() {
            Ok(s) => s,
            Err(_) => return ready(Err(AuthError::NotAuthenticated)),
        };

        let user_id = match session.get::<i32>("user_id") {
            Ok(Some(id)) => id,
            _ => return ready(Err(AuthError::NotAuthenticated)),
        };

        let app_state = match req.app_data::<Data<AppState>>() {
            Some(s) => s,
            None => return ready(Err(AuthError::Internal("Could not get app state".into()))),
        };

        let mut conn = match app_state.db_pool.get() {
            Ok(c) => c,
            Err(_) => {
                return ready(Err(AuthError::Internal(
                    "Could not connect to the database".into(),
                )));
            }
        };

        let _user = match get_user(&mut conn, user_id) {
            Ok(u) => {
                if u.is_active {
                    log::warn!("Got User {}", user_id);
                    u
                } else {
                    log::warn!("User {} is not active", user_id);
                    return ready(Err(AuthError::NotAuthenticated));
                }
            }
            Err(e) => {
                log::trace!("User not Authenticated {:?}", e);
                return ready(Err(AuthError::NotAuthenticated));
            }
        };

        let memberships = match load_roles(&mut conn, user_id) {
            Ok(r) => r,
            Err(_e) => return ready(Err(AuthError::Internal("Could not load user roles".into()))),
        };

        ready(Ok(AuthContext {
            user_id,
            memberships,
        }))
    }
}

fn load_roles(
    conn: &mut SqliteConnection,
    user_id: i32,
) -> Result<Vec<MembershipContext>, AppError> {
    use crate::schema::{memberships, roles};

    roles::table
        .inner_join(memberships::table.on(memberships::role_id.eq(roles::id)))
        .filter(memberships::user_id.eq(user_id))
        .filter(memberships::active.eq(true))
        .select((memberships::host_id, roles::name))
        .load::<MembershipContext>(conn)
        .map_err(|e| AppError::Db(e))
}

fn load_roles_user(conn: &mut SqliteConnection, user_id: i32) -> Result<Vec<MemberRole>, AppError> {
    use crate::schema::{memberships, roles};

    roles::table
        .inner_join(memberships::table.on(memberships::role_id.eq(roles::id)))
        .filter(memberships::user_id.eq(user_id))
        .filter(memberships::active.eq(true))
        .select(roles::name)
        .load::<MemberRole>(conn)
        .map_err(|e| AppError::Db(e))
}

pub fn require_role(
    user_roles: &[MemberRole],
    required_roles: &[MemberRole],
) -> Result<(), AuthError> {
    if has_role(user_roles, required_roles) {
        Ok(())
    } else {
        Err(AuthError::Forbidden("Insufficient role"))
    }
}

pub fn require_role_for_host(
    context: &AuthContext,
    host_id: i32,
    required_roles: &[MemberRole],
) -> Result<(), AuthError> {
    let has_required_role = context.memberships.iter().any(|membership| {
        membership.host_id == host_id && required_roles.contains(&membership.role)
    });

    if has_required_role {
        Ok(())
    } else {
        Err(AuthError::Forbidden("Insufficient role for this host"))
    }
}


pub fn has_role(user_roles: &[MemberRole], required_roles: &[MemberRole]) -> bool {
    required_roles.iter().any(|r| user_roles.contains(r))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MemberRole;

    #[test]
    fn has_role_returns_true_when_user_has_role() {
        //let user = test_user(vec![MemberRole::Admin]);
        let user_roles = vec![MemberRole::Admin];

        assert!(has_role(&user_roles, &[MemberRole::Admin]));
    }

    #[test]
    fn has_role_returns_false_when_user_lacks_role() {
        //let user = test_user(vec![MemberRole::Member]);
        let user_roles = vec![MemberRole::Member];

        assert!(!has_role(&user_roles, &[MemberRole::Admin]));
    }

    #[test]
    fn has_role_accepts_any_of_multiple_roles() {
        //let user = test_user(vec![MemberRole::Reviewer]);
        let user_roles = vec![MemberRole::Reviewer];

        assert!(has_role(
            &user_roles,
            &[MemberRole::Admin, MemberRole::Reviewer]
        ));
    }

    #[test]
    fn require_role_returns_ok_when_allowed() {
        //let user = test_user(vec![MemberRole::Admin]);
        let user_roles = vec![MemberRole::Admin];

        assert!(require_role(&user_roles, &[MemberRole::Admin]).is_ok());
    }

    #[test]
    fn require_role_returns_forbidden_when_denied() {
        //let user = test_user(vec![MemberRole::Member]);
        let user_roles = vec![MemberRole::Member];

        let err = require_role(&user_roles, &[MemberRole::Admin]).unwrap_err();

        matches!(err, AuthError::Forbidden(_));
    }
}
