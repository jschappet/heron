-- This file should undo anything in `up.sql`
drop index if exists idx_recipe_draft_on_user_id;
drop index if exists idx_recipe_drafts_status;
drop table if exists recipe_drafts;
