CREATE TABLE entities (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    host_id INTEGER NOT NULL,
    entity_type TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE,
    UNIQUE (host_id, name)
);

CREATE INDEX idx_entities_type ON entities(entity_type);