-- This file should undo anything in `up.sql`
DROP TABLE memberships;
drop INDEX if EXISTS idx_memberships_unique_active_role;
DROP INDEX if EXISTS idx_memberships_user_id;
DROP INDEX if EXISTS idx_memberships_role_id;
DROP INDEX if EXISTS idx_memberships_active;    