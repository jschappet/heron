-- Your SQL goes here
-- Add profile picture to users
ALTER TABLE users ADD COLUMN profile_picture TEXT;

-- Add offers table
CREATE TABLE if not EXISTS offers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    offer TEXT NOT NULL,
    request TEXT NOT NULL,
    location TEXT,
    contact_link TEXT,
    start_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_date TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add completed_offers table
CREATE TABLE if not exists completed_offers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    offer_id INTEGER NOT NULL REFERENCES offers(id) ON DELETE CASCADE,
    reviewer_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER, -- optional numeric rating
    review TEXT,   -- optional textual review
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes for faster lookups
CREATE INDEX if not exists idx_offers_user_id ON offers(user_id);
CREATE INDEX if not exists idx_completed_offers_offer_id ON completed_offers(offer_id);
CREATE INDEX if not exists idx_completed_offers_reviewer_id ON completed_offers(reviewer_id);
