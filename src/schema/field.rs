use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[forge::forge_enum]
pub enum FieldValueType {
    Text,
    Enum,
    Bool,
    Int,
    Decimal,
    List,
    Url,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FieldDefinition {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: String,
    pub value_type: FieldValueType,
    pub color: Option<String>,
    pub options: Option<Vec<String>>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskField {
    pub id: Uuid,
    pub task_id: Uuid,
    pub field_id: Uuid,
    pub value: String,
}
