-- Your SQL goes here



CREATE TABLE memberships_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    host_id INTEGER NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);


INSERT INTO memberships_new (
    id,user_id,role_id,host_id,active,created_at,ended_at
)
SELECT
    id,user_id,role_id,2,active,created_at,ended_at
FROM memberships;


DROP TABLE memberships;

ALTER TABLE memberships_new RENAME TO memberships;

CREATE UNIQUE INDEX IF NOT EXISTS idx_memberships_unique_active_role
ON memberships(user_id, host_id, role_id, active);
CREATE INDEX IF NOT EXISTS idx_memberships_user_id ON memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_memberships_role_id ON memberships(role_id);
CREATE INDEX IF NOT EXISTS idx_memberships_active ON memberships(active);
