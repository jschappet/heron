-- Your SQL goes here
-- Migration: Add registration_id to ticket and enforce uniqueness on (user_id, event_id)

-- 1. Rename the old table
ALTER TABLE ticket RENAME TO ticket_old;

-- 2. Create the new table with registration_id foreign key
CREATE TABLE ticket (
    id TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    event_id TEXT NOT NULL,
    registration_id INTEGER,
    checked_in TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    FOREIGN KEY (registration_id) REFERENCES registration(id) ON DELETE SET NULL
);

-- 3. Copy data over (null registration_id for existing data)
INSERT INTO ticket (id, user_id, event_id, checked_in, created_at)
SELECT id, user_id, event_id, checked_in, created_at FROM ticket_old;

-- 4. Drop the old table
DROP TABLE ticket_old;
