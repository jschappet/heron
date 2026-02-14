-- Your SQL goes here
CREATE TABLE hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    slug TEXT NOT NULL UNIQUE,          -- 'regenerateskagit', 'revillagesociety'
    host_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    active BOOLEAN DEFAULT 0 NOT NULL
);
INSERT INTO hosts (id,slug, host_name, display_name, base_url, active) VALUES(0,'unknown','unknown','Unknown Host','', false);
INSERT INTO hosts (slug, host_name, display_name, base_url, active) VALUES
('regenerateskagit', 'regenerateskagit.org', 'Regenerate Skagit', 'https://regenerateskagit.org', 1),
('dev', 'dev.regenerateskagit.org', '[DEV] Regenerate Skagit', 'https://dev.regenerateskagit.org', 1),
('revillagesociety', 'revillagesociety.org', 'Revillage Society', 'https://revillagesociety.org', 1)
