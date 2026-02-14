-- ============================================
-- rating_events: append-only log of every rating
-- ============================================

CREATE TABLE rating_events (
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    rating_type   TEXT NOT NULL,      -- e.g. 'recipe'
    target_id     TEXT NOT NULL,      -- slug, url, or UUID
    user_id       INTEGER,            -- optional, null = anonymous
    rating        INTEGER NOT NULL,   -- 1–5 for now
    review        TEXT,               -- optional human text
    rating_details TEXT,              -- json or misc future metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP

);

-- Helpful composite index for lookups
CREATE INDEX idx_rating_events_type_target
    ON rating_events (rating_type, target_id);

-- Useful if you want to query all ratings by a user later
CREATE INDEX idx_rating_events_user
    ON rating_events (user_id);


-- ============================================
-- rating_summary: snapshot aggregated values
-- ============================================

CREATE TABLE rating_summary (
    rating_type    TEXT NOT NULL,
    target_id      TEXT NOT NULL,
    rating_sum     INTEGER NOT NULL,
    rating_count   INTEGER NOT NULL,
    average_rating REAL NOT NULL,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (rating_type, target_id)
);

-- Index to speed filtering if you ever aggregate by type
CREATE INDEX idx_rating_summary_type
    ON rating_summary (rating_type);
