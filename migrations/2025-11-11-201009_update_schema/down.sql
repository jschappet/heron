-- This file should undo anything in `up.sql`
-- Rollback completed_offers and offers tables
DROP TABLE IF EXISTS completed_offers;
DROP TABLE IF EXISTS offers;

-- Remove profile_picture from users (SQLite cannot DROP column directly)
-- If you want a rollback for profile_picture, you would need to recreate the table without that column
-- ALTER TABLE users DROP COLUMN profile_picture; -- SQLite does not support this
