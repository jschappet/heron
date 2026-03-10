-- This file should undo anything in `up.sql`
-- 001_drop_regen_vital_signs_views_down.sql
-- Migration rollback: drop all regenerative vital signs views

DROP VIEW IF EXISTS v_regen_vital_signs;
DROP VIEW IF EXISTS v_active_projects_30d;
DROP VIEW IF EXISTS v_total_hours_30d;
DROP VIEW IF EXISTS v_active_contributors_30d;
DROP VIEW IF EXISTS v_dollars_available;
DROP VIEW IF EXISTS v_entity_balances;