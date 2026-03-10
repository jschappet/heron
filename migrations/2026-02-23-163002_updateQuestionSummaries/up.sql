-- Your SQL goes here
PRAGMA foreign_keys=off;

CREATE TABLE question_summaries_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    question_uuid TEXT NOT NULL UNIQUE,
    answers_count INTEGER NOT NULL,
    question_text TEXT NOT NULL,
    summary TEXT NOT NULL,
    prompt TEXT NOT NULL,
    created_at TIMESTAMP
);

INSERT INTO question_summaries_new (
    id,
    question_uuid,
    answers_count,
    question_text,
    summary,
    prompt,
    created_at
)
SELECT
    id,
    question_uuid,
    answers_count,
    question_text,
    summary,
    prompt,
    created_at
FROM question_summaries;

DROP TABLE question_summaries;

ALTER TABLE question_summaries_new
RENAME TO question_summaries;



CREATE TABLE weekly_answers_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    question_uuid TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO weekly_answers_new (id, name, email, question_uuid, answer, created_at)
SELECT id, name, email, question_uuid, answer, created_at
FROM weekly_answers;

DROP TABLE weekly_answers;

ALTER TABLE weekly_answers_new RENAME TO weekly_answers;

PRAGMA foreign_keys=on;
