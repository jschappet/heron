# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Is

Heron is the shared infrastructure platform for ReVillage Society and Regenerate Skagit — a **place-based platform** (not SaaS) built around multi-tenancy via "landscapes" (called `hosts`). It manages singular identity across related landscapes, memberships/roles, events, skills, drafts, and email communication.

## Commands

```bash
cargo build              # Build
cargo build --release    # Optimized build
cargo run                # Run dev server (localhost:8582)
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
```

Database migrations run automatically at startup. To create a new migration:
```bash
diesel migration generate <name>
```

## Architecture

### Multi-Tenancy via Hosts
Every user-facing query is scoped by `host_id`. The `HostMiddleware` extracts the current landscape from the request. RBAC is enforced per-host via `MemberRole` (Public → Member → Reviewer → Admin). Auth context is available in handlers via `AuthContext` extracted from session.

### Request Flow
`Actix-web` → middleware stack (`HostMiddleware`, `AdminMiddleware`, `IdentityMiddleware`) → route handler → domain/service → Diesel + SQLite

### Layer Structure
- **`routes/`** — Actix-web handlers. Group endpoints by feature. Routes are registered in `routes/mod.rs` with a static `ROUTES` registry for audit logging.
- **`domains/`** — Stateful domain objects injected as `web::Data` in `main.rs`. Coordinate business logic across services. Current domains: `LedgerDomain`, `MemberDomain`, `DraftDomain`, `WeeklyReflectionDomain`.
- **`services/`** — Stateless functions doing DB operations. Called by domains and routes.
- **`models/`** — Diesel `Queryable`/`Insertable`/`Updateable` structs, one file per entity. Schema auto-generated from migrations into `schema.rs`.
- **`middleware/`** — `HostMiddleware` (multi-tenancy), `AdminMiddleware` (route guarding).
- **`errors/`** — `AppError` and `AuthError` enums with `ResponseError` impls. Use `?` with `From` conversions throughout.
- **`validator.rs`** — Input validation and role/permission checking helpers.
- **`app_state.rs`** — `AppState` struct holding `db_pool`, handlebars registry, and settings.
- **`settings.rs`** — Config loading: `config/application.toml` → `config/{RUN_MODE}.toml` → `APP_*` env vars.

### Database
SQLite via Diesel 2.x with R2D2 connection pooling. `PRAGMA foreign_keys = ON` is enforced at connection time. Migrations are embedded and auto-run at startup via `diesel_migrations`.



### Key Patterns
- **Templating**: Handlebars for server-side HTML rendering
- **Email**: Lettre via SMTP (configured in `[smtp]` section)
- **Sessions**: Cookie-based via `actix-session` (not server-side)
- **Static files**: Served from `./webroot/` only in debug builds
- **Config**: Multi-file via `config` crate; add new settings as a struct in `settings.rs`

### Front-end Web Page

The frontend web pages for this project heron are built in other projects: 
- ReVillage Socitety - ../revillage-society 
- Regenerate Skagit - ../regen-skagit/

Both of these projects use Project Heron as a backend system. 

These systems currently share copied content. TODO: Plan for future Refactor these into content delivered by Project Heron.

## Configuration

Copy `config/application.toml` for local overrides. The app reads `config/{RUN_MODE}.toml` where `RUN_MODE` defaults to `development`. Sensitive values (SMTP credentials, cookie keys, API keys) live in this file and are not committed.



## Types & Enums (CRITICAL)

Strong typing is a core design principle of this project. Types must clearly separate concerns between database, domain logic, and API.

---

### Type Layers

We maintain **three distinct type layers**:

#### 1. Database Models (Diesel)

* Located in `src/models/`
* Used only for DB interaction
* Derive Diesel traits (`Queryable`, `Insertable`, etc.)
* May include internal-only fields

#### 2. Domain Types

* Located in `src/services/` or `src/domain/`
* Represent business logic concepts
* Independent of HTTP and Diesel
* Prefer rich enums over primitive types

#### 3. API DTOs (serde)

* Located in `src/routes/` 
* Used for request/response bodies
* Must be stable and explicit
* Never expose DB models directly

---

### Enum Usage (IMPORTANT)

Enums are preferred over strings or integers for all state and variant logic.

#### Rules:

* Use enums for status, type, and variant fields
* Avoid "stringly typed" logic
* Keep enum definitions centralized and reusable

Example:

```rust
enum OrderStatus {
    Pending,
    Completed,
    Cancelled,
}
```

---

### Diesel + SQLite Enum Strategy

SQLite does not support native enums.

Therefore:

* Store enums as `TEXT` in the database
* Use Diesel `AsExpression` + `FromSqlRow` or manual conversions
* Map DB values ↔ Rust enums explicitly

Pattern:

* DB layer uses strings
* Domain layer uses enums
* Conversion happens in DB or service layer

---

### Serialization (API Layer)

* All API-facing enums must derive:

  * `Serialize`
  * `Deserialize`

* Use stable, explicit naming:

```rust
#[serde(rename_all = "snake_case")]
```

* Never rely on implicit enum serialization format

---

### Conversions (REQUIRED)

Use explicit conversions between layers:

* `From` / `Into`
* or `TryFrom` for fallible mappings

Example:

```rust
impl From<DbUser> for ApiUser { ... }
```

Avoid:

* Inline conversion logic inside handlers
* Duplicated mapping code

---

### ID Types

* Prefer strong ID types over raw integers/strings when meaningful

Example:

```rust
struct UserId(i32);
```

* Avoid passing raw IDs across layers when domain meaning exists

---

### Nullability

* Use `Option<T>` only when absence is meaningful
* Do not overuse `Option` to avoid modeling decisions

---

### Anti-Patterns (DO NOT DO)

* ❌ Returning Diesel structs directly in API responses
* ❌ Using `String` instead of enums for state
* ❌ Duplicating enum definitions across layers without mapping
* ❌ Implicit conversions between DB and API types
* ❌ Embedding business logic inside DTOs

---

### When Adding New Types

1. Decide which layer(s) the type belongs to
2. Define enums for all finite states
3. Add explicit conversions between layers
4. Ensure API serialization is stable
5. Keep DB representation simple (SQLite-friendly)


---

### Memory File

This project has a memory stored in MEMORY.md

It is a linked list of all the memories stored a /memories/

Please review these as you are working with this code.
