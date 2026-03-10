-- Your SQL goes here
-- 001_create_regen_vital_signs_views.sql
-- Migration: create views for regenerative dashboard (current available)

-- 1. Base balance view: computes net balance per entity and resource_type
CREATE VIEW IF NOT EXISTS v_entity_balances AS
SELECT
    entity_id,
    resource_type,
    SUM(amount) AS balance
FROM (
    SELECT to_entity AS entity_id,
           resource_type,
           quantity_value AS amount
    FROM flow_events
    UNION ALL
    SELECT from_entity AS entity_id,
           resource_type,
           -quantity_value AS amount
    FROM flow_events
)
GROUP BY entity_id, resource_type;

-- 2. Total dollars available across all entities
CREATE VIEW IF NOT EXISTS v_dollars_available AS
SELECT
    IFNULL(SUM(balance), 0) AS dollars_available
FROM v_entity_balances
WHERE resource_type = 'Dollars'
 AND balance > 0;

-- 3. Active contributors in last 30 days
CREATE VIEW IF NOT EXISTS v_active_contributors_30d AS
SELECT
    COUNT(DISTINCT from_entity) AS active_contributors
FROM flow_events
WHERE resource_type = 'labor_time'
  AND timestamp >= datetime('now', '-30 days');

-- 4. Total labor hours in last 30 days
CREATE VIEW IF NOT EXISTS v_total_hours_30d AS
SELECT
    IFNULL(SUM(quantity_value), 0) AS total_hours
FROM flow_events
WHERE resource_type = 'labor_time'
  AND timestamp >= datetime('now', '-30 days');

-- 5. Active projects drawing resources in last 30 days
CREATE VIEW IF NOT EXISTS v_active_projects_30d AS
SELECT
    COUNT(DISTINCT e.id) AS active_projects
FROM entities e
JOIN flow_events f
  ON f.to_entity = e.id
WHERE e.entity_type in ('project','team','organization') 
  AND f.timestamp >= datetime('now', '-30 days');

-- 6. Unified vital signs view for dashboard
CREATE VIEW IF NOT EXISTS v_regen_vital_signs AS
SELECT
    (SELECT dollars_available FROM v_dollars_available) AS dollars_available,
    (SELECT active_contributors FROM v_active_contributors_30d) AS active_contributors_30d,
    (SELECT total_hours FROM v_total_hours_30d) AS total_hours_30d,
    (SELECT active_projects FROM v_active_projects_30d) AS active_projects_30d;



-- Indexes to speed up v_entity_balances and v_regen_vital_signs views

-- 1. Index on the source of flows, per resource type
CREATE INDEX IF NOT EXISTS idx_flow_from_entity_resource
ON flow_events(from_entity, resource_type);

-- 2. Index on the destination of flows, per resource type
CREATE INDEX IF NOT EXISTS idx_flow_to_entity_resource
ON flow_events(to_entity, resource_type);

-- 3. Index on timestamp (helps last-30-days queries)
CREATE INDEX IF NOT EXISTS idx_flow_timestamp
ON flow_events(timestamp);

-- 4. Composite index on to_entity + timestamp (speeds up active_projects queries)
CREATE INDEX IF NOT EXISTS idx_flow_to_entity_timestamp
ON flow_events(to_entity, timestamp);

-- 5. Optional: Composite index on resource_type + timestamp (speeds up labor_time aggregates)
CREATE INDEX IF NOT EXISTS idx_flow_resource_timestamp
ON flow_events(resource_type, timestamp);

-- 6. Ensure entities table has an index on entity_type for filtering projects/teams/orgs
CREATE INDEX IF NOT EXISTS idx_entities_type
ON entities(entity_type);