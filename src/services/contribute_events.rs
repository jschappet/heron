use crate::db::{DbConn, DbPool};

use crate::errors::app_error::AppError;
use crate::models::contribution::{ContributionEvent, ContributionEventInput};
use crate::models::effort_context::EffortContextInput;
use crate::routes::config::ConfigHash;
use crate::schema::contribution_events::dsl::*;
use crate::types::Audience;
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};



#[derive(Clone)]
pub struct ContributionDomain {
    service: ContributionEventsService,
}

impl ContributionDomain {
    pub fn new(pool: DbPool) -> Self {
        Self {
            service: ContributionEventsService::new(pool),
        }
    }

    pub fn create_event(
        &self,
        payload: NewContributionEvent,
    ) -> Result<ContributionEvent, AppError> {
        self.service.create_from_payload(payload)
    }

    pub fn get_effort_contexts(&self, audience: Audience) -> Result<Vec<ConfigHash>, AppError> {
        self.service.get_effort_contexts(audience)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewContributionEvent {
    pub context_id: Option<String>,
    pub context_name: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub availability_days: Option<String>,
    pub availability_times: Option<String>,
    pub notes: Option<String>,
    pub effort: Option<String>,
}

#[derive(Clone)]
pub struct ContributionEventsService {
    db_pool: DbPool,
}

impl ContributionEventsService {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub fn db_conn(&self) -> Result<DbConn, AppError> {
        self.db_pool
            .get()
            .map_err(|err| AppError::User(err.to_string()))
    }

    pub fn create_from_payload(
        &self,
        payload: NewContributionEvent,
    ) -> Result<ContributionEvent, AppError> {
        let mut conn = self.db_conn()?;

        let resolved_context_id = match payload.context_id {
            Some(ref short_code) => self.get_id_from_short_code(&mut conn, short_code)?,
            None => self.get_id_or_create_context(
                &mut conn,
                payload
                    .context_name
                    .as_ref()
                    .ok_or(AppError::BadRequest("Missing context_name".to_string()))?,
            )?,
        };

        let contrib_id = self.get_id_or_create_contributor(
            &mut conn,
            &payload.name.clone().unwrap_or_else(|| "Anonymous".into()),
            &payload
                .email
                .clone()
                .unwrap_or_else(|| "blank@nobody.com".into()),
        )?;

        let event_input = ContributionEventInput {
            context_id: resolved_context_id,
            contributor_id: contrib_id,
            effort_date: Some(Utc::now().naive_utc()),
            hours: None,
            work_done: payload.effort.clone().unwrap_or_default(),
            details: payload.notes,
            appreciation_message: Some(String::new()),
            public_flag: Some(false),
        };

        self.create_event(&mut conn, &event_input)
    }

    fn create_event(
        &self,
        conn: &mut diesel::SqliteConnection,
        event: &ContributionEventInput,
    ) -> Result<ContributionEvent, AppError> {
        diesel::insert_into(contribution_events)
            .values(event)
            .execute(conn)
            .map_err(AppError::Db)?;

        contribution_events
            .order(id.desc())
            .first::<ContributionEvent>(conn)
            .map_err(AppError::Db)
    }

    fn get_id_from_short_code(
        &self,
        conn: &mut diesel::SqliteConnection,
        short_code_str: &str,
    ) -> Result<i32, AppError> {
        use crate::schema::effort_contexts::dsl::*;
        let existing = effort_contexts
            .filter(short_code.eq(short_code_str))
            .select(id)
            .first::<i32>(conn)
            .optional()
            .map_err(AppError::Db)?;

        match existing {
            Some(existing_id) => Ok(existing_id),
            None => self.create_new_context(conn, short_code_str),
        }
    }

    fn get_id_or_create_context(
        &self,
        conn: &mut diesel::SqliteConnection,
        context_name_str: &str,
    ) -> Result<i32, AppError> {
        use crate::schema::effort_contexts::dsl::*;
        let existing = effort_contexts
            .filter(name.eq(context_name_str))
            .or_filter(short_code.eq(context_name_str))
            .select((id, name))
            .first::<(i32, String)>(conn)
            .optional()
            .map_err(AppError::Db)?;

        match existing {
            Some((new_id, _)) => Ok(new_id),
            None => self.create_new_context(conn, context_name_str),
        }
    }

    fn get_id_or_create_contributor(
        &self,
        conn: &mut diesel::SqliteConnection,
        contrib_name: &str,
        contrib_email: &str,
    ) -> Result<i32, AppError> {
        use crate::schema::contributors::dsl::*;
        let existing = contributors
            .filter(email.eq(contrib_email))
            .select((id, name))
            .first::<(i32, Option<String>)>(conn)
            .optional()
            .map_err(AppError::Db)?;

        if let Some((new_id, _)) = existing {
            Ok(new_id)
        } else {
            diesel::insert_into(contributors)
                .values((name.eq(contrib_name), email.eq(contrib_email)))
                .execute(conn)
                .map_err(AppError::Db)?;

            contributors
                .order(id.desc())
                .select(id)
                .first(conn)
                .map_err(AppError::Db)
        }
    }

    fn create_new_context(
        &self,
        conn: &mut diesel::SqliteConnection,
        context_name: &str,
    ) -> Result<i32, AppError> {
        use crate::schema::effort_contexts::dsl::*;
        let new_context = EffortContextInput {
            name: context_name.to_string(),
            context_type: "general".to_string(),
            description: "User-contributed effort context".to_string(),
            short_code: context_name_to_short_code(context_name),
            active_flag: false,
        };

        diesel::insert_into(effort_contexts)
            .values(&new_context)
            .execute(conn)
            .map_err(AppError::Db)?;

        effort_contexts
            .order(id.desc())
            .select(id)
            .first(conn)
            .map_err(AppError::Db)
    }

    pub fn get_effort_contexts(&self, audience: Audience) -> Result<Vec<ConfigHash>, AppError> {
        let mut conn = self.db_conn()?;

        use crate::schema::effort_contexts::dsl::*;

        let mut query = effort_contexts
            .order(name.asc())
            .select((short_code, name))
            .into_boxed(); // <-- important

        
        match audience {
            Audience::Admin => {}
            _ => query = query.filter(active_flag.eq(true)),
        }
        
        let contexts = query
            .load::<(String, String)>(&mut conn)
            .map_err(AppError::Db)?
            .into_iter()
            .map(|(key, value)| ConfigHash { key, value })
            .collect();

        Ok(contexts)
    }
}

fn context_name_to_short_code(context_name: &str) -> String {
    let vowels = ['A', 'E', 'I', 'O', 'U'];
    context_name
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .filter(|c| !vowels.contains(c))
        .take(8)
        .collect()
}
