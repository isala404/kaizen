use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const USER_COLUMNS: &str = "id, email, name, password_hash, created_at, updated_at";

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub(crate) struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PublicUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            name: u.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

impl From<User> for Viewer {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
        }
    }
}
