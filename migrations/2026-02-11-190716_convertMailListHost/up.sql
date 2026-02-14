-- Your SQL goes here
-- ==========================================
-- Migration: Add host_id to mailing_list_subscribers
-- ==========================================
--BEGIN TRANSACTION;
-- 1️⃣ Add the new column (nullable for now)
ALTER TABLE mailing_list_subscribers
ADD COLUMN host_id INTEGER;
-- 2️⃣ Backfill host_id from hosts table
UPDATE mailing_list_subscribers
SET host_id = (
    SELECT id
    FROM hosts
    WHERE mailing_list_subscribers.host = hosts.host_name
);
-- Verify backfill
-- SELECT COUNT(*) FROM mailing_list_subscribers WHERE host_id IS NULL;
-- Should return 0
-- 3️⃣ Create a new table with proper NOT NULL and foreign key constraint
CREATE TABLE mailing_list_subscribers_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    host_id INTEGER NOT NULL REFERENCES hosts(id),
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    confirmation_token TEXT,
    unsubscribed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);
-- 4️⃣ Copy data from old table into new table
INSERT INTO mailing_list_subscribers_new (
    id, host_id, name, email, confirmed,
    confirmation_token, unsubscribed, created_at
)
SELECT
    id, host_id, name, email, confirmed,
    confirmation_token, unsubscribed, created_at
FROM mailing_list_subscribers;
-- 5️⃣ Drop old table
DROP TABLE mailing_list_subscribers;
-- 6️⃣ Rename new table
ALTER TABLE mailing_list_subscribers_new
RENAME TO mailing_list_subscribers;
-- 7️⃣ Recreate indexes
CREATE UNIQUE INDEX idx_mailing_list_unique_per_host
ON mailing_list_subscribers(host_id, email);
CREATE INDEX idx_mailing_list_subscribers_confirmed
ON mailing_list_subscribers(confirmed);
--COMMIT;