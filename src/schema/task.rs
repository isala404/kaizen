use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TASK_COLUMNS: &str = "id, user_id, title, description, status, time_spent_secs, position, due_at, created_at, updated_at";

#[forge::forge_enum]
pub enum TaskStatus {
    Inbox,
    UpNext,
    InProgress,
    Focused,
    Paused,
    Done,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub time_spent_secs: i64,
    pub position: i32,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
