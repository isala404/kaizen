use forge::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    schema::{TASK_FIELD_COLUMNS, TaskField},
    support::{qualify_columns, require_field_definition_for_user, require_task_for_user},
};

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
    let task_field_columns = qualify_columns("tf", TASK_FIELD_COLUMNS);
    let query = format!(
        "SELECT {task_field_columns} FROM task_fields tf JOIN tasks t ON t.id = tf.task_id WHERE tf.task_id = $1 AND t.user_id = $2"
    );

    let fields = sqlx::query_as::<_, TaskField>(&query)
        .bind(input.task_id)
        .bind(user_id)
        .fetch_all(ctx.db())
        .await?;
    Ok(fields)
}

#[forge::query]
pub async fn list_all_task_fields(ctx: &QueryContext) -> Result<Vec<TaskField>> {
    let user_id = ctx.user_id()?;
    let task_field_columns = qualify_columns("tf", TASK_FIELD_COLUMNS);
    let query = format!(
        "SELECT {task_field_columns} FROM task_fields tf JOIN tasks t ON t.id = tf.task_id WHERE t.user_id = $1"
    );

    let fields = sqlx::query_as::<_, TaskField>(&query)
        .bind(user_id)
        .fetch_all(ctx.db())
        .await?;
    Ok(fields)
}

#[forge::mutation]
pub async fn set_task_field(ctx: &MutationContext, input: SetTaskFieldInput) -> Result<TaskField> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    require_task_for_user(&mut *conn, input.task_id, user_id).await?;
    require_field_definition_for_user(&mut *conn, input.field_id, user_id).await?;

    let upsert_task_field_query = format!(
        "INSERT INTO task_fields (task_id, field_id, value) VALUES ($1, $2, $3) ON CONFLICT (task_id, field_id) DO UPDATE SET value = EXCLUDED.value RETURNING {TASK_FIELD_COLUMNS}"
    );

    let tf = sqlx::query_as::<_, TaskField>(&upsert_task_field_query)
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
