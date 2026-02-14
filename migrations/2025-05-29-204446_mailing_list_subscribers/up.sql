CREATE TABLE mailing_list_subscribers (
    id            INTEGER PRIMARY KEY AUTOINCREMENT  NOT NULL,
    name          TEXT NOT NULL,
    email         TEXT NOT NULL UNIQUE,
    confirmed     BOOLEAN DEFAULT FALSE not null,
    confirmation_token TEXT,
    unsubscribed  BOOLEAN DEFAULT FALSE not null,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_mailing_list_subscribers_email ON mailing_list_subscribers(email);
CREATE INDEX idx_mailing_list_subscribers_confirmed ON mailing_list_subscribers(confirmed);
