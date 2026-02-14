-- Your SQL goes here
-- up.sql
CREATE TABLE weekly_answers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    question_uuid TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- up.sql
CREATE TABLE question_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_uuid TEXT NOT NULL UNIQUE,
    answers_count INTEGER NOT NULL,
    question_text TEXT NOT NULL,
    summary TEXT NOT NULL,
    prompt TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
