Reference Doc: Domain vs Service in Rust

Here’s a cheat sheet / reference you can pin for when you’re building future domains like UserDomain or DraftDomain.

Rust Backend Domain & Service Reference
1. Route Layer

Responsibilities:

Parse HTTP input (web::Json, web::Query, etc.)

Authenticate / authorize if needed

Call domain with business intent

Return HTTP response

Should NOT:

Talk to DB directly

Build domain objects manually

Contain orchestration logic

Example:

#[post("/users")]
pub async fn create_user(
    users: web::Data<UserDomain>,
    payload: web::Json<NewUser>,
) -> Result<HttpResponse, AppError> {
    let user = users.create_user(payload.into_inner())?;
    Ok(HttpResponse::Ok().json(user))
}

2. Domain Layer

Responsibilities:

Expose business capabilities for a slice of your app

Orchestrate one or more services to fulfill the capability

Enforce rules / defaults / invariants

Keep routes thin

Should NOT:

Directly perform raw DB queries

Know about HTTP or routing

Often a wrapper around one or more services:

#[derive(Clone)]
pub struct UserDomain {
    service: UserService,
}

impl UserDomain {
    pub fn new(pool: DbPool) -> Self {
        Self { service: UserService::new(pool) }
    }

    pub fn create_user(&self, payload: NewUser) -> Result<User, AppError> {
        self.service.create_from_payload(payload)
    }
}

3. Service Layer

Responsibilities:

Implement actual behavior

Talk to repositories / DB / external systems

Return results to the domain

Should NOT:

Know about HTTP

Make decisions across multiple services (that’s domain’s job)

```
#[derive(Clone)]
pub struct UserService {
    db_pool: DbPool,
}

impl UserService {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub fn create_from_payload(&self, payload: NewUser) -> Result<User, AppError> {
        let mut conn = self.db_pool.get().map_err(|e| AppError::Db(e.into()))?;
        // DB logic here
    }
}
```

4. Repositories (Optional / Future)

Responsibilities:

Pure DB access, simple CRUD

Called by Services

Helps decouple DB logic from behavior

Example: UserRepository, ContributionRepository

5. Flow Summary
Route (HTTP) 
   │
   ▼
Domain (Capability / Orchestration) 
   │
   ▼
Service (Behavior Implementation) 
   │
   ▼
Repository / DB / External


Route = input/output adapter

Domain = control panel / orchestrator

Service = engine / worker

Repository = storage / access layer

Tips / Gotchas

One connection per orchestration call

Pass &mut conn through helpers. Avoid multiple db_pool.get() inside sub-functions.

Variable naming matters

Avoid shadowing outer variable names in match blocks, especially with ?.

e.g., resolved_context_id instead of context_id.

Keep routes thin

Any logic that touches multiple tables, default values, or orchestration → domain.

Domain boundaries are conceptual, not physical

They can live in same file as service until complexity grows.

Service = DB + behavior; Domain = capability + orchestration

If your service starts doing orchestration → split into domain.