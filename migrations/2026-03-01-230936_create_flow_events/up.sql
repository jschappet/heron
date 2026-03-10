CREATE TABLE flow_events (
    id TEXT PRIMARY KEY NOT NULL,

    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    from_entity TEXT NOT NULL,
    to_entity TEXT NOT NULL,
    host_id INTEGER NOT NULL,

    resource_type TEXT NOT NULL,
    quantity_value REAL NOT NULL,
    quantity_unit TEXT NOT NULL,

    notes TEXT,

    FOREIGN KEY (from_entity)
        REFERENCES entities(id)
        ON DELETE RESTRICT,

    FOREIGN KEY (to_entity)
        REFERENCES entities(id)
        ON DELETE RESTRICT,

    FOREIGN KEY (host_id)
        REFERENCES hosts(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_flow_timestamp ON flow_events(timestamp);
CREATE INDEX idx_flow_from_entity ON flow_events(from_entity);
CREATE INDEX idx_flow_to_entity ON flow_events(to_entity);
CREATE INDEX idx_flow_host_id ON flow_events(host_id);
CREATE INDEX idx_flow_resource_time ON flow_events(resource_type, timestamp);
CREATE INDEX idx_flow_host_time ON flow_events(host_id, timestamp);