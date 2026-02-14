-- contribution_events existed but was empty in PROD.
-- We are redefining its purpose and structure.
DROP TABLE IF EXISTS contribution_events;


-- Contributors: people who have offered effort,
-- whether or not they have an account yet
CREATE TABLE contributors (
  id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name          TEXT,
  email         TEXT,
  user_id       INTEGER,
  created_at    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Contexts where effort lands:
-- projects, events, organizations, internal work, etc.
CREATE TABLE effort_contexts (
  id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  context_type   TEXT NOT NULL,
  short_code    TEXT UNIQUE NOT NULL,
  name          TEXT NOT NULL,
  description   TEXT NOT NULL,
  created_at    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  active_flag   BOOLEAN DEFAULT 0 NOT NULL

);

-- Expressions of effort themselves
CREATE TABLE contribution_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

  context_id INTEGER NOT NULL,
  contributor_id INTEGER NOT NULL,

  effort_date DATETIME,
  hours       REAL,

  work_done   TEXT DEFAULT '' NOT NULL,
  details     TEXT DEFAULT '{}' NOT NULL,

  appreciation_message TEXT  DEFAULT '' NOT NULL,
  public_flag BOOLEAN DEFAULT 0 NOT NULL,

  created_at  DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

  FOREIGN KEY (context_id) REFERENCES effort_contexts(id),
  FOREIGN KEY (contributor_id) REFERENCES contributors(id)
);

-- Helpful indexes for future dashboards (cheap now, valuable later)
CREATE INDEX idx_contribution_events_context
  ON contribution_events(context_id);

CREATE INDEX idx_contribution_events_contributor
  ON contribution_events(contributor_id);

CREATE INDEX idx_contributors_email
  ON contributors(email);
