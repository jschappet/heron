CREATE TABLE user_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    token_hash TEXT NOT NULL,
    purpose TEXT NOT NULL, -- 'verify_account' | 'reset_password'
    expires_at DATETIME NOT NULL,
    used_at DATETIME,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

ALTER TABLE users ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT 0;


CREATE UNIQUE INDEX idx_user_tokens_unique_active
ON user_tokens (user_id, purpose)
WHERE used_at IS NULL;

