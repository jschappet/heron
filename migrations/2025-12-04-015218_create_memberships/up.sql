-- Your SQL goes here
CREATE TABLE memberships (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

-- Each user should not have two active memberships for the same role
CREATE UNIQUE INDEX idx_memberships_unique_active_role
ON memberships(user_id, role_id, active);

-- Common join patterns
CREATE INDEX idx_memberships_user_id ON memberships(user_id);
CREATE INDEX idx_memberships_role_id ON memberships(role_id);
CREATE INDEX idx_memberships_active ON memberships(active);
