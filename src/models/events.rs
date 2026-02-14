
use diesel::{prelude::*};
use diesel::sqlite::SqliteConnection;
use chrono::NaiveDateTime;
use crate::schema::events::dsl::events;
use crate::schema::events::*;


use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::events)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub location: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::events)]
pub struct NewEvent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub location: String,
}

pub fn create_event(
    conn: &mut SqliteConnection,
    new_event: NewEvent,
) -> QueryResult<Event> {
    diesel::insert_into(events)
        .values(&new_event)
        .execute(conn)?;

    events.order(created_at.desc()).first::<Event>(conn)
}

pub fn get_events(conn: &mut SqliteConnection) -> QueryResult<Vec<Event>> {
    events.order(start_time.asc()).load::<Event>(conn)
}

pub fn get_event(conn: &mut SqliteConnection, event_id: String) -> QueryResult<Event> {
    events.find(event_id).first::<Event>(conn)
}

pub fn update_event(
    conn: &mut SqliteConnection,
    event_id: String,
    updated_event: NewEvent,
) -> QueryResult<Event> {
    diesel::update(events.find(event_id))
        .set((
            name.eq(updated_event.name),
            description.eq(updated_event.description),
            start_time.eq(updated_event.start_time),
            end_time.eq(updated_event.end_time),
            location.eq(updated_event.location),
        ))
        .execute(conn)?;

    events.find(updated_event.id).first::<Event>(conn)
}

pub fn delete_event(conn: &mut SqliteConnection, event_id: String) -> QueryResult<usize> {
    diesel::delete(events.find(event_id)).execute(conn)
}
