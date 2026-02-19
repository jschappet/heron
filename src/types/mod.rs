use std::fmt::Display;

use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::{Sqlite, SqliteValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

mod field_schema;
pub(crate) mod method;

pub use field_schema::{ FrontendSchema, load_frontend_schema};
mod auth_context;
//pub use auth_context::{AdminContext, MembershipContext};



// Wrapper type
#[derive(Debug, Clone, PartialEq, AsExpression, FromSqlRow, Deserialize, Serialize, Default)]
#[diesel(sql_type = Text)]
pub struct JsonField(pub JsonValue);

// FromSql for reading from SQLite TEXT
impl FromSql<Text, Sqlite> for JsonField {
    fn from_sql(mut bytes: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        let s = bytes.read_text(); // <-- use as_str() instead of value()
        let json = serde_json::from_str(s)?;
        Ok(JsonField(json))
    }
}

// ToSql for writing to SQLite TEXT
impl ToSql<Text, Sqlite> for JsonField {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let s = serde_json::to_string(&self.0)?;
        out.set_value(s); // <-- use set_value() instead of write_all()
        Ok(IsNull::No)
    }
}

// Convenience conversions
impl From<JsonValue> for JsonField {
    fn from(value: JsonValue) -> Self {
        JsonField(value)
    }
}

impl From<JsonField> for JsonValue {
    fn from(field: JsonField) -> Self {
        field.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RatingType {
    Recipe,
    Article,
    Reciprocity,
}

impl From<&str> for RatingType {
    fn from(s: &str) -> Self {
        match s {
            "recipe" => RatingType::Recipe,
            "article" => RatingType::Article,
            "reciprocity" => RatingType::Reciprocity,
            _ => panic!("Unknown rating type"),
        }
    }
}

impl RatingType {
    pub fn as_str(&self) -> &str {
        match self {
            RatingType::Recipe => "recipe",
            RatingType::Article => "article",
            RatingType::Reciprocity => "reciprocity",
        }
    }
}

impl Display for RatingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ToSql<Text, Sqlite> for RatingType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let s = match self {
            RatingType::Recipe => "recipe",
            RatingType::Article => "article",
            RatingType::Reciprocity => "reciprocity",
        };
        out.set_value(s);
        Ok(IsNull::No)
    }
}

#[derive(Serialize)]
pub struct ConfigOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    fn meta(self) -> (&'static str, &'static str) {
        match self {
            Difficulty::Easy => ("easy", "Easy"),
            Difficulty::Medium => ("medium", "Medium"),
            Difficulty::Hard => ("hard", "Hard"),
        }
    }

    pub fn value(self) -> &'static str {
        self.meta().0
    }

    pub fn label(self) -> &'static str {
        self.meta().1
    }

    pub fn all() -> Vec<ConfigOption> {
        [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
            .into_iter()
            .map(|d| ConfigOption {
                value: d.value(),
                label: d.label(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Dietary {
    GlutenFree,
    NonDairy,
    Vegetarian,
    Vegan,
}

impl Dietary {
    fn meta(self) -> (&'static str, &'static str) {
        match self {
            Dietary::GlutenFree => ("gluten-free", "Gluten-Free"),
            Dietary::NonDairy => ("non-dairy", "Non-Dairy"),
            Dietary::Vegetarian => ("vegetarian", "Vegetarian"),
            Dietary::Vegan => ("vegan", "Vegan"),
        }
    }

    pub fn value(self) -> &'static str {
        self.meta().0
    }

    pub fn label(self) -> &'static str {
        self.meta().1
    }

    pub fn all() -> Vec<ConfigOption> {
        [
            Dietary::GlutenFree,
            Dietary::NonDairy,
            Dietary::Vegetarian,
            Dietary::Vegan,
        ]
        .into_iter()
        .map(|d| ConfigOption {
            value: d.value(),
            label: d.label(),
        })
        .collect()
    }
}

#[derive(Debug, Clone,Copy,PartialEq,Eq,AsExpression,Serialize,Deserialize,FromSqlRow)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]

pub enum DraftStatus {
    Draft,
    Approved,
    Submitted,
    Pending,
    ChangesRequested,
    Rejected,
    Deployed,
}

impl DraftStatus {
    fn meta(self) -> (&'static str, &'static str) {
        match self {
            DraftStatus::Draft => ("draft", "Draft"),
            DraftStatus::Submitted => ("submitted", "Submitted"),
            DraftStatus::Approved => ("approved", "Approved"),
            DraftStatus::Pending => ("pending", "Pending"),
            DraftStatus::ChangesRequested => ("changes_requested", "Changes Requested"),
            DraftStatus::Rejected => ("rejected", "Rejected"),
            DraftStatus::Deployed => ("deployed", "Deployed"),
        }
    }

    pub fn value(self) -> &'static str {
        self.meta().0
    }

    pub fn label(self) -> &'static str {
        self.meta().1
    }

    pub fn all() -> Vec<ConfigOption> {
        [
            DraftStatus::Draft,
            DraftStatus::Submitted,
            DraftStatus::Approved,
            DraftStatus::Pending,
            DraftStatus::ChangesRequested,
            DraftStatus::Rejected,
            DraftStatus::Deployed,
        ]
        .into_iter()
        .map(|s| ConfigOption {
            value: s.value(),
            label: s.label(),
        })
        .collect()
    }
}



impl FromSql<Text, Sqlite> for DraftStatus {
    fn from_sql(mut bytes: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        //let s = std::str::from_utf8(bytes.as_bytes())?;
        let s = bytes.read_text();
        match s {
            "draft" => Ok(DraftStatus::Draft),
            "approved" => Ok(DraftStatus::Approved),
            "changes_requested" => Ok(DraftStatus::ChangesRequested),
            "deployed" => Ok(DraftStatus::Deployed),
            "pending" => Ok(DraftStatus::Pending),
            "rejected" => Ok(DraftStatus::Rejected),
            "submitted" => Ok(DraftStatus::Submitted),
            other => Err(format!("Unknown DraftStatus value: {}", other).into()),
        }
    }
}

impl ToSql<Text, Sqlite> for DraftStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.value());
        Ok(IsNull::No)
    }
}


#[derive(Debug, Clone,Copy,PartialEq,Eq,Serialize,Deserialize,FromSqlRow)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Public,
    Admin,
    Reviewer,
    Member,
    Organizer,
    Volunteer,
    Guest,
}
impl MemberRole {
    fn meta(self) -> (&'static str, &'static str) {
        match self {
            MemberRole::Public => ("public", "Public"),
            MemberRole::Admin => ("admin", "Admin"),
            MemberRole::Reviewer => ("reviewer", "Reviewer"),
            MemberRole::Member => ("member", "Member"),
            MemberRole::Organizer => ("organizer", "Organizer"),
            MemberRole::Volunteer => ("volunteer", "Volunteer"),
            MemberRole::Guest => ("guest", "Guest"),
        }
    }

    pub fn value(self) -> &'static str {
        self.meta().0
    }

    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        self.meta().1
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<ConfigOption> {
        [MemberRole::Public,MemberRole::Admin, MemberRole::Reviewer, MemberRole::Member, MemberRole::Organizer, MemberRole::Volunteer, MemberRole::Guest]
            .into_iter()
            .map(|d| ConfigOption {
                value: d.value(),
                label: d.label(),
            })
            .collect()
    }
}


impl FromSql<Text, Sqlite> for MemberRole {
    fn from_sql(mut bytes: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        //let s = std::str::from_utf8(bytes.as_bytes())?;
        let s = bytes.read_text();
        match s {
            "admin" => Ok(MemberRole::Admin),
            "reviewer" => Ok(MemberRole::Reviewer),
            "member" => Ok(MemberRole::Member),
            "organizer" => Ok(MemberRole::Organizer),
            "volunteer" => Ok(MemberRole::Volunteer),
            "guest" => Ok(MemberRole::Guest),
            other => Err(format!("Unknown MemberRole value: {}", other).into()),
        }
    }
}

impl ToSql<Text, Sqlite> for MemberRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.value());
        Ok(IsNull::No)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, Serialize, Deserialize, FromSqlRow)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]
pub enum TokenPurpose {
    VerifyAccount,
    ResetPassword,
    ChangeEmail,
}

impl TokenPurpose {
    fn meta(self) -> (&'static str, &'static str) {
        match self {
            TokenPurpose::VerifyAccount => ("verify_account", "Verify Account"),
            TokenPurpose::ResetPassword => ("reset_password", "Reset Password"),
            TokenPurpose::ChangeEmail => ("change_email", "Change Email"),
        }
    }

    pub fn value(self) -> &'static str {
        self.meta().0
    }

    pub fn label(self) -> &'static str {
        self.meta().1
    }

    pub fn all() -> Vec<ConfigOption> {
        [
            TokenPurpose::VerifyAccount,
            TokenPurpose::ResetPassword,
            TokenPurpose::ChangeEmail,
        ]
        .into_iter()
        .map(|p| ConfigOption {
            value: p.value(),
            label: p.label(),
        })
        .collect()
    }
}




impl FromSql<Text, Sqlite> for TokenPurpose {
    fn from_sql(mut bytes: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        let s = bytes.read_text();
        match s {
            "verify_account" => Ok(TokenPurpose::VerifyAccount),
            "reset_password" => Ok(TokenPurpose::ResetPassword),
            "change_email" => Ok(TokenPurpose::ChangeEmail),
            other => Err(format!("Unknown TokenPurpose value: {}", other).into()),
        }
    }
}


impl ToSql<Text, Sqlite> for TokenPurpose {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.value());
        Ok(IsNull::No)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, 
    Serialize, Deserialize, FromSqlRow, Default, Hash)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    Recipe,
    #[default]
    Post,
    Event,
    Organization,
    Page,
}

impl DocType {
    fn meta(self) -> (&'static str, &'static str) {
        match self {
            DocType::Recipe => ("recipe", "Recipe"),
            DocType::Post => ("post", "Post"),
            DocType::Event => ("event", "Event"),
            DocType::Organization => ("organization", "Organization"),
            DocType::Page => ("page", "Page"),
        }
    }

    pub fn value(self) -> &'static str {
        self.meta().0
    }

    pub fn label(self) -> &'static str {
        self.meta().1
    }

    pub fn all() -> Vec<ConfigOption> {
        [
            DocType::Recipe,
            DocType::Post,
            DocType::Event,
            DocType::Organization,
            DocType::Page,
        ]
        .into_iter()
        .map(|d| ConfigOption {
            value: d.value(),
            label: d.label(),
        })
        .collect()
    }

    pub fn list() -> Vec<&'static str> {
        [
            DocType::Recipe,
            DocType::Post,
            DocType::Event,
            DocType::Organization,
            DocType::Page,
        ]
        .into_iter()
        .map(|d|  d.value())
        .collect()
    }
}


impl FromSql<Text, Sqlite> for DocType {
    fn from_sql(mut bytes: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        let s = bytes.read_text();
        match s {
            "recipe" => Ok(DocType::Recipe),
            "post" => Ok(DocType::Post),
            "event" => Ok(DocType::Event),
            "organization" => Ok(DocType::Organization),
            "page" => Ok(DocType::Page),
            other => Err(format!("Unknown DocType value: {}", other).into()),
        }
    }
}

impl ToSql<Text, Sqlite> for DocType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.value());
        Ok(IsNull::No)
    }
}

pub enum Audience {
    Public,
    Authenticated,
    Owner,
    Admin
}

pub enum Role {
    User,
    Admin,
    SuperAdmin,
}
