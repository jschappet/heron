-- Your SQL goes here
CREATE TABLE if not exists recipe_drafts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    tags TEXT NOT NULL,          -- comma-separated
    author TEXT NOT NULL,
    prep_time INTEGER,           -- minutes
    cook_time INTEGER,           -- minutes
    total_time INTEGER,          -- optional, can compute
    servings INTEGER,
    difficulty TEXT,             -- enum: easy, medium, hard
    source TEXT,                 -- new field for source/inspiration
    dietary TEXT,                -- JSON array string ["gluten-free","vegan"]
    body_md TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',  -- draft | submitted | changes_requested | approved
    submitted_by INTEGER NOT NULL,         -- user_id
    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    reviewed_by INTEGER,                   -- user_id
    reviewed_at DATETIME,
    review_notes TEXT,
    details TEXT                           -- JSON blob for future fields
);
CREATE INDEX idx_recipe_drafts_status ON recipe_drafts(status);
CREATE INDEX idx_recipe_drafts_submitted_by ON recipe_drafts(submitted_by);