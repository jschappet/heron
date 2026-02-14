-- Your SQL goes here
-- Generic drafts table for all document types
CREATE TABLE IF NOT EXISTS drafts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

    doc_type TEXT NOT NULL,              -- e.g. 'recipe', 'post', 'event', 'organization'
    title TEXT NOT NULL,
    description TEXT,

    tags TEXT,                           -- comma-separated
    author TEXT,

    -- Structured but optional metadata (recipe-specific, event-specific, etc.)
    meta TEXT,                           -- JSON blob (prep_time, cook_time, servings, etc.)

    body_md TEXT NOT NULL,               -- primary content

    status TEXT NOT NULL DEFAULT 'draft',  -- draft | submitted | changes_requested | approved

    submitted_by INTEGER NOT NULL,         -- user_id
    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    reviewed_by INTEGER,                  -- user_id
    reviewed_at DATETIME,
    review_notes TEXT,

    details TEXT                          -- JSON blob for future expansion / overrides
);

-- Indexes for workflow performance
CREATE INDEX idx_drafts_doc_type ON drafts(doc_type);
CREATE INDEX idx_drafts_status ON drafts(status);
CREATE INDEX idx_drafts_submitted_by ON drafts(submitted_by);
