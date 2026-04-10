-- This file should undo anything in `up.sql`
-- Drop indexes (safe even if they don't exist depending on Diesel config)
DROP INDEX IF EXISTS idx_entities_type_id;

DROP INDEX IF EXISTS idx_entity_relationship_unique;
DROP INDEX IF EXISTS idx_entity_relationships_host;

DROP INDEX IF EXISTS idx_entity_identities_primary;
DROP INDEX IF EXISTS idx_entity_identities_lookup;

-- Drop tables (order matters due to FKs)
DROP TABLE IF EXISTS entity_relationships;
DROP TABLE IF EXISTS entity_identities;
DROP TABLE IF EXISTS entity_types;
DROP TABLE IF EXISTS entity_merges;