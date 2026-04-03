use forge::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::TaskField;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTaskFieldInput {
    pub task_id: Uuid,
    pub field_id: Uuid,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTaskFieldInput {
    pub task_id: Uuid,
    pub field_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTaskFieldsInput {
    pub task_id: Uuid,
}

#[forge::query]
pub async fn list_task_fields(
    ctx: &QueryContext,
    input: ListTaskFieldsInput,
) -> Result<Vec<TaskField>> {
    let user_id = ctx.user_id()?;
    // Verify task ownership through join
    let fields = sqlx::query_as::<_, TaskField>(
        "SELECT tf.id, tf.task_id, tf.field_id, tf.value
         FROM task_fields tf
         JOIN tasks t ON t.id = tf.task_id
         WHERE tf.task_id = $1 AND t.user_id = $2",
    )
    .bind(input.task_id)
    .bind(user_id)
    .fetch_all(ctx.db())
    .await?;
    Ok(fields)
}

#[forge::query]
pub async fn list_all_task_fields(ctx: &QueryContext) -> Result<Vec<TaskField>> {
    let user_id = ctx.user_id()?;
    let fields = sqlx::query_as::<_, TaskField>(
        "SELECT tf.id, tf.task_id, tf.field_id, tf.value
         FROM task_fields tf
         JOIN tasks t ON t.id = tf.task_id
         WHERE t.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(ctx.db())
    .await?;
    Ok(fields)
}

#[forge::mutation]
pub async fn set_task_field(ctx: &MutationContext, input: SetTaskFieldInput) -> Result<TaskField> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    // Verify task ownership
    let task_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tasks WHERE id = $1 AND user_id = $2)")
            .bind(input.task_id)
            .bind(user_id)
            .fetch_one(&mut conn)
            .await?;

    if !task_exists {
        return Err(ForgeError::NotFound("Task not found".into()));
    }

    // Verify field definition ownership
    let field_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM field_definitions WHERE id = $1 AND user_id = $2)",
    )
    .bind(input.field_id)
    .bind(user_id)
    .fetch_one(&mut conn)
    .await?;

    if !field_exists {
        return Err(ForgeError::NotFound("Field definition not found".into()));
    }

    let tf = sqlx::query_as::<_, TaskField>(
        "INSERT INTO task_fields (task_id, field_id, value)
         VALUES ($1, $2, $3)
         ON CONFLICT (task_id, field_id) DO UPDATE SET value = EXCLUDED.value
         RETURNING id, task_id, field_id, value",
    )
    .bind(input.task_id)
    .bind(input.field_id)
    .bind(&input.value)
    .fetch_one(&mut conn)
    .await?;

    Ok(tf)
}

#[forge::mutation]
pub async fn remove_task_field(ctx: &MutationContext, input: RemoveTaskFieldInput) -> Result<()> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let rows = sqlx::query(
        "DELETE FROM task_fields tf
         USING tasks t
         WHERE tf.task_id = t.id AND tf.task_id = $1 AND tf.field_id = $2 AND t.user_id = $3",
    )
    .bind(input.task_id)
    .bind(input.field_id)
    .bind(user_id)
    .execute(&mut conn)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(ForgeError::NotFound("Task field not found".into()));
    }
    Ok(())
}
