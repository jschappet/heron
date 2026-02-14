-- This file should undo anything in `up.sql`
drop table if exists drafts;
drop index if exists idx_drafts_doc_type;
drop index if exists idx_drafts_status;
drop index if exists idx_drafts_submitted_by;