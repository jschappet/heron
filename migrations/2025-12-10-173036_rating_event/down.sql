-- This file should undo anything in `up.sql`
drop index if exists idx_rating_events_type_target;
drop index if exists idx_rating_events_user;

drop table if exists rating_events;
drop table if exists rating_summary;    

