use chrono::NaiveDateTime;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use uuid::Uuid;

use crate::schema::flow_events::{self, BoxedQuery};

pub type FlowQueryBox<'a> = BoxedQuery<'a, diesel::sqlite::Sqlite>;


pub enum FlowDirection {
    From,
    To,
    Both,
}

pub struct FlowQuery {
    pub host: i32,
    pub entity: Option<Uuid>,
    pub direction: FlowDirection,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl FlowQuery {
    pub fn new(host: i32) -> Self {
        Self {
            host,
            entity: None,
            direction: FlowDirection::Both,
            since: None,
            until: None,
            limit: None,
            offset: None,
        }
    }

    pub fn entity(mut self, id: Uuid) -> Self {
        self.entity = Some(id);
        self
    }

    pub fn from(mut self) -> Self {
        self.direction = FlowDirection::From;
        self
    }

    pub fn to(mut self) -> Self {
        self.direction = FlowDirection::To;
        self
    }

    pub fn both(mut self) -> Self {
        self.direction = FlowDirection::Both;
        self
    }

    pub fn since(mut self, t: NaiveDateTime) -> Self {
        self.since = Some(t);
        self
    }

    pub fn until(mut self, t: NaiveDateTime) -> Self {
        self.until = Some(t);
        self
    }

    pub fn limit(mut self, n: i64) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: i64) -> Self {
        self.offset = Some(n);
        self
    }
}

impl FlowQuery {
    pub fn apply<'a>(
        &self,
        mut query: FlowQueryBox<'a>,
    ) -> FlowQueryBox<'a> {

        query = query.filter(flow_events::host_id.eq(self.host));
        
        if let Some(entity) = self.entity {
            match self.direction {
                FlowDirection::From => {
                    query = query.filter(flow_events::from_entity.eq(entity.to_string()));
                }
                FlowDirection::To => {
                    query = query.filter(flow_events::to_entity.eq(entity.to_string()));
                }
                FlowDirection::Both => {
                    query = query.filter(
                        flow_events::from_entity
                            .eq(entity.to_string())
                            .or(flow_events::to_entity.eq(entity.to_string())),
                    );
                }
            }
        }

        if let Some(since) = self.since {
            query = query.filter(flow_events::timestamp.ge(since));
        }

        if let Some(until) = self.until {
            query = query.filter(flow_events::timestamp.le(until));
        }

        if let Some(limit) = self.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = self.offset {
            query = query.offset(offset);
        }

        query
    }
}


