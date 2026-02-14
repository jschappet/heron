CREATE TABLE sms_replies (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    registration_id INTEGER REFERENCES registration(id) ON DELETE SET NULL,
    to_number TEXT NOT NULL,
    from_number TEXT NOT NULL,
    body TEXT NOT NULL,
    received_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    parsed_response TEXT,
    raw_payload TEXT -- optional: store as serialized JSON string
);
