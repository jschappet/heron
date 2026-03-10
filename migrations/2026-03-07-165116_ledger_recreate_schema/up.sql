-- Your SQL goes here
-- ============================================================
-- LEDGER SCHEMA RESET / MIGRATION
-- Assumes test data only. Safe to rebuild schema.
-- SQLite / Diesel compatible
-- ============================================================


-- ============================================================
-- DROP EXISTING TABLES
-- ============================================================

DROP TABLE IF EXISTS flow_actions;
DROP TABLE IF EXISTS flow_events;
DROP TABLE IF EXISTS entity_users;
DROP TABLE IF EXISTS entity_aliases;
DROP TABLE IF EXISTS entities;


-- ============================================================
-- ENTITIES
-- Actors in the economic graph (people, orgs, teams, events)
-- ============================================================

CREATE TABLE entities (
    id TEXT PRIMARY KEY NOT NULL,

    name TEXT NOT NULL,
    entity_type TEXT NOT NULL,

    host_id INTEGER NOT NULL,

    created_by TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    details TEXT DEFAULT '{}' NOT NULL,

    FOREIGN KEY (host_id)
        REFERENCES hosts(id)
        ON DELETE CASCADE,

    UNIQUE (host_id, name)
);

CREATE INDEX idx_entities_type
ON entities(entity_type);



-- ============================================================
-- ENTITY ALIASES
-- Alternative names for entities
-- ============================================================

CREATE TABLE entity_aliases (
    id TEXT PRIMARY KEY NOT NULL,

    entity_id TEXT NOT NULL,
    alias TEXT NOT NULL,

    created_by TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

    FOREIGN KEY(entity_id)
        REFERENCES entities(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_entity_alias
ON entity_aliases(alias);



-- ============================================================
-- USER ↔ ENTITY RELATIONSHIP
-- Connects authentication accounts to economic actors
-- ============================================================

CREATE TABLE entity_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

    entity_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,

    role TEXT DEFAULT 'member' NOT NULL,
    status TEXT DEFAULT 'active' NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

    FOREIGN KEY(entity_id) REFERENCES entities(id),
    FOREIGN KEY(user_id) REFERENCES users(id),

    UNIQUE(entity_id, user_id)
);



-- ============================================================
-- FLOW EVENTS
-- Immutable ledger entries
-- ============================================================

CREATE TABLE flow_events (
    id TEXT PRIMARY KEY NOT NULL,

    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    from_entity TEXT NOT NULL,
    to_entity TEXT NOT NULL,

    host_id INTEGER NOT NULL,

    resource_type TEXT NOT NULL,
    quantity_value REAL NOT NULL,
    quantity_unit TEXT NOT NULL,

    notes TEXT,
    details TEXT DEFAULT '{}' NOT NULL,

    created_by TEXT NOT NULL,

    CHECK (from_entity != to_entity),

    FOREIGN KEY (from_entity)
        REFERENCES entities(id)
        ON DELETE RESTRICT,

    FOREIGN KEY (to_entity)
        REFERENCES entities(id)
        ON DELETE RESTRICT,

    FOREIGN KEY (host_id)
        REFERENCES hosts(id)
        ON DELETE CASCADE
);


-- ============================================================
-- FLOW INDEXES
-- ============================================================

CREATE INDEX idx_flow_timestamp
ON flow_events(timestamp);

CREATE INDEX idx_flow_from_entity
ON flow_events(from_entity);

CREATE INDEX idx_flow_to_entity
ON flow_events(to_entity);

CREATE INDEX idx_flow_host_id
ON flow_events(host_id);

CREATE INDEX idx_flow_resource_time
ON flow_events(resource_type, timestamp);

CREATE INDEX idx_flow_host_time
ON flow_events(host_id, timestamp);

CREATE INDEX idx_flow_from_entity_resource
ON flow_events(from_entity, resource_type);

CREATE INDEX idx_flow_to_entity_resource
ON flow_events(to_entity, resource_type);

CREATE INDEX idx_flow_to_entity_timestamp
ON flow_events(to_entity, timestamp);

CREATE INDEX idx_flow_resource_timestamp
ON flow_events(resource_type, timestamp);



-- ============================================================
-- FLOW ACTIONS
-- Validation, dispute, expiration, comments
-- ============================================================

CREATE TABLE flow_actions (
    id TEXT PRIMARY KEY NOT NULL,

    flow_id TEXT NOT NULL,
    action_type TEXT NOT NULL,

    actor_entity TEXT NOT NULL,

    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

    details TEXT DEFAULT '{}' NOT NULL,

    FOREIGN KEY(flow_id)
        REFERENCES flow_events(id)
        ON DELETE CASCADE,

    FOREIGN KEY(actor_entity)
        REFERENCES entities(id)
);

CREATE INDEX idx_flow_actions_flow
ON flow_actions(flow_id);



-- ============================================================
-- APPEND-ONLY LEDGER ENFORCEMENT
-- Prevent updates or deletes on flow_events
-- ============================================================

CREATE TRIGGER prevent_flow_update
BEFORE UPDATE ON flow_events
BEGIN
    SELECT RAISE(FAIL, 'flow_events are append-only');
END;

CREATE TRIGGER prevent_flow_delete
BEFORE DELETE ON flow_events
BEGIN
    SELECT RAISE(FAIL, 'flow_events cannot be deleted');
END;