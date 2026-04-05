use forge::prelude::*;
use uuid::Uuid;

pub const POSITION_STEP: i32 = 10_000;

pub fn next_position(max_position: Option<i32>) -> i32 {
    max_position.unwrap_or_default() + POSITION_STEP
}

pub fn required_trimmed(value: &str, message: &'static str) -> Result<String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(ForgeError::Validation(message.into()));
    }

    Ok(normalized)
}

pub async fn require_task_for_user<'a, E>(executor: E, task_id: Uuid, user_id: Uuid) -> Result<()>
where
    E: sqlx::PgExecutor<'a>,
{
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM tasks WHERE id = $1 AND user_id = $2)",
    )
    .bind(task_id)
    .bind(user_id)
    .fetch_one(executor)
    .await?;

    if !exists {
        return Err(ForgeError::NotFound("Task not found".into()));
    }

    Ok(())
}

pub async fn require_field_definition_for_user<'a, E>(
    executor: E,
    field_id: Uuid,
    user_id: Uuid,
) -> Result<()>
where
    E: sqlx::PgExecutor<'a>,
{
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM field_definitions WHERE id = $1 AND user_id = $2)",
    )
    .bind(field_id)
    .bind(user_id)
    .fetch_one(executor)
    .await?;

    if !exists {
        return Err(ForgeError::NotFound("Field definition not found".into()));
    }

    Ok(())
}
