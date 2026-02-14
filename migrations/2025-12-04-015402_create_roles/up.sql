-- Your SQL goes here
CREATE TABLE roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    show_in_directory BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Helpful index for filtering by visibility
CREATE INDEX idx_roles_show_in_directory ON roles(show_in_directory);
