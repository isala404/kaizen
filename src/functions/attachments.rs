use forge::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::Attachment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttachmentsInput {
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachmentInput {
    pub task_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAttachmentInput {
    pub id: Uuid,
}

#[forge::query]
pub async fn list_attachments(
    ctx: &QueryContext,
    input: ListAttachmentsInput,
) -> Result<Vec<Attachment>> {
    let user_id = ctx.user_id()?;
    let attachments = sqlx::query_as::<_, Attachment>(
        "SELECT a.id, a.task_id, a.user_id, a.filename, a.content_type,
                a.size_bytes, a.storage_key, a.display, a.created_at
         FROM attachments a
         JOIN tasks t ON t.id = a.task_id
         WHERE a.task_id = $1 AND t.user_id = $2
         ORDER BY a.created_at ASC",
    )
    .bind(input.task_id)
    .bind(user_id)
    .fetch_all(ctx.db())
    .await?;
    Ok(attachments)
}

#[forge::mutation]
pub async fn create_attachment(
    ctx: &MutationContext,
    input: CreateAttachmentInput,
) -> Result<Attachment> {
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

    let storage_key = format!("attachments/{}/{}", input.task_id, Uuid::new_v4());

    let attachment = sqlx::query_as::<_, Attachment>(
        "INSERT INTO attachments (task_id, user_id, filename, content_type, size_bytes, storage_key)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, task_id, user_id, filename, content_type, size_bytes, storage_key, display, created_at",
    )
    .bind(input.task_id)
    .bind(user_id)
    .bind(&input.filename)
    .bind(&input.content_type)
    .bind(input.size_bytes)
    .bind(&storage_key)
    .fetch_one(&mut conn)
    .await?;

    Ok(attachment)
}

#[forge::mutation]
pub async fn delete_attachment(ctx: &MutationContext, input: DeleteAttachmentInput) -> Result<()> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let rows = sqlx::query("DELETE FROM attachments WHERE id = $1 AND user_id = $2")
        .bind(input.id)
        .bind(user_id)
        .execute(&mut conn)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ForgeError::NotFound("Attachment not found".into()));
    }
    Ok(())
}
