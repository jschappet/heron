-- This file should undo anything in `up.sql`
-- Rollback: remove the unique index and registration_id column from ticket

-- Turn off foreign key checks
PRAGMA foreign_keys = OFF;

-- Rename the old ticket table
ALTER TABLE ticket RENAME TO ticket_old;

-- Recreate the ticket table without registration_id and unique constraint
CREATE TABLE ticket (
    id TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    event_id TEXT NOT NULL,
    checked_in TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);

-- Copy the data back (excluding registration_id)
INSERT INTO ticket (id, user_id, event_id, checked_in, created_at)
SELECT id, user_id, event_id, checked_in, created_at FROM ticket_old;

-- Drop the old table
DROP TABLE ticket_old;

-- Turn foreign key checks back on
PRAGMA foreign_keys = ON;
