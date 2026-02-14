-- Your SQL goes here
CREATE TABLE if not EXISTS wants_to_contribute (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    offer_id INTEGER NOT NULL,
    helper_user_id INTEGER NOT NULL,
    who TEXT,
    how_helping TEXT,
    availability_days TEXT,
    availability_times TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (offer_id) REFERENCES offers(id),
    FOREIGN KEY (helper_user_id) REFERENCES users(id)
);


CREATE TABLE if not EXISTS contribution_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    offer_id INTEGER NOT NULL,
    helper_user_id INTEGER NOT NULL,
    who TEXT,
    help_timestamp DATETIME,
    work_done TEXT,
    appreciation_message TEXT,
    public_flag BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (offer_id) REFERENCES offers(id),
    FOREIGN KEY (helper_user_id) REFERENCES users(id)
);
