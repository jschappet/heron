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

## Configuration

Copy `config/application.toml` for local overrides. The app reads `config/{RUN_MODE}.toml` where `RUN_MODE` defaults to `development`. Sensitive values (SMTP credentials, cookie keys, API keys) live in this file and are not committed.
