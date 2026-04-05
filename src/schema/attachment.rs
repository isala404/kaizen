use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ATTACHMENT_COLUMNS: &str =
    "id, task_id, user_id, filename, content_type, size_bytes, storage_key, display, created_at";

#[forge::forge_enum]
pub enum AttachmentDisplay {
    Inline,
    Attached,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_key: String,
    pub display: AttachmentDisplay,
    pub created_at: DateTime<Utc>,
}
