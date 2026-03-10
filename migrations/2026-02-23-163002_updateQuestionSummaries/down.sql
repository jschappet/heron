-- This file should undo anything in `up.sql`
PRAGMA foreign_keys=off;

CREATE TABLE question_summaries_old (
    id INTEGER PRIMARY KEY,
    question_uuid TEXT NOT NULL UNIQUE,
    answers_count INTEGER NOT NULL,
    question_text TEXT NOT NULL,
    summary TEXT NOT NULL,
    prompt TEXT NOT NULL,
    created_at TIMESTAMP
);

INSERT INTO question_summaries_old
SELECT * FROM question_summaries;

DROP TABLE question_summaries;

ALTER TABLE question_summaries_old
RENAME TO question_summaries;

PRAGMA foreign_keys=on;
