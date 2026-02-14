-- This file should undo anything in `up.sql`
-- Rollback in dependency order
DROP INDEX IF EXISTS idx_contributors_email;
DROP INDEX IF EXISTS idx_contribution_events_contributor;
DROP INDEX IF EXISTS idx_contribution_events_context;

DROP TABLE IF EXISTS contribution_events;
DROP TABLE IF EXISTS effort_contexts;
DROP TABLE IF EXISTS contributors;
