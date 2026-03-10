-- This file should undo anything in `up.sql`
-- ============================================================
-- LEDGER SCHEMA ROLLBACK
-- Reverses the migration defined in up.sql
-- ============================================================


-- ============================================================
-- DROP TRIGGERS
-- ============================================================

DROP TRIGGER IF EXISTS prevent_flow_update;
DROP TRIGGER IF EXISTS prevent_flow_delete;


-- ============================================================
-- DROP INDEXES (FLOW EVENTS)
-- ============================================================

DROP INDEX IF EXISTS idx_flow_timestamp;
DROP INDEX IF EXISTS idx_flow_from_entity;
DROP INDEX IF EXISTS idx_flow_to_entity;
DROP INDEX IF EXISTS idx_flow_host_id;
DROP INDEX IF EXISTS idx_flow_resource_time;
DROP INDEX IF EXISTS idx_flow_host_time;
DROP INDEX IF EXISTS idx_flow_from_entity_resource;
DROP INDEX IF EXISTS idx_flow_to_entity_resource;
DROP INDEX IF EXISTS idx_flow_to_entity_timestamp;
DROP INDEX IF EXISTS idx_flow_resource_timestamp;


-- ============================================================
-- DROP INDEXES (OTHER TABLES)
-- ============================================================

DROP INDEX IF EXISTS idx_flow_actions_flow;
DROP INDEX IF EXISTS idx_entity_alias;
DROP INDEX IF EXISTS idx_entities_type;


-- ============================================================
-- DROP TABLES (reverse dependency order)
-- ============================================================

DROP TABLE IF EXISTS flow_actions;
DROP TABLE IF EXISTS flow_events;
DROP TABLE IF EXISTS entity_users;
DROP TABLE IF EXISTS entity_aliases;
DROP TABLE IF EXISTS entities;