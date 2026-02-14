-- Your SQL goes here
ALTER TABLE mailing_list_subscribers
ADD COLUMN host TEXT NOT NULL DEFAULT 'default-host';