//! Database models for Diesel ORM
//!
//! These models are separate from domain entities to maintain layer separation.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use super::schema::users;

/// Database model for users table (for querying)
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbUser {
    pub id: Uuid,
    pub email: String,
    pub hashed_password: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New user model for insertion
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewDbUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub hashed_password: &'a str,
    pub display_name: Option<&'a str>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
