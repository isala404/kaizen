use chrono::{DateTime, Utc};
use forge::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{Task, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub due_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderTaskInput {
    pub id: Uuid,
    pub status: TaskStatus,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTaskInput {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTaskInput {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskInput {
    pub id: Uuid,
}

#[forge::query]
pub async fn list_tasks(ctx: &QueryContext) -> Result<Vec<Task>> {
    let user_id = ctx.user_id()?;
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT id, user_id, title, description, status, time_spent_secs,
                position, due_at, created_at, updated_at
         FROM tasks
         WHERE user_id = $1
         ORDER BY position ASC, created_at ASC",
    )
    .bind(user_id)
    .fetch_all(ctx.db())
    .await?;

    Ok(tasks)
}

#[forge::query]
pub async fn get_task(ctx: &QueryContext, input: GetTaskInput) -> Result<Task> {
    let user_id = ctx.user_id()?;
    sqlx::query_as::<_, Task>(
        "SELECT id, user_id, title, description, status, time_spent_secs,
                position, due_at, created_at, updated_at
         FROM tasks
         WHERE id = $1 AND user_id = $2",
    )
    .bind(input.id)
    .bind(user_id)
    .fetch_optional(ctx.db())
    .await?
    .ok_or_else(|| ForgeError::NotFound("Task not found".into()))
}

#[forge::mutation]
pub async fn create_task(ctx: &MutationContext, input: CreateTaskInput) -> Result<Task> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ForgeError::Validation("Title is required".into()));
    }

    let user_id = ctx.user_id()?;
    let status = input.status.unwrap_or(TaskStatus::Inbox);
    let description = input.description.unwrap_or_default();

    let mut conn = ctx.conn().await?;

    let max_pos: Option<i32> =
        sqlx::query_scalar("SELECT MAX(position) FROM tasks WHERE user_id = $1 AND status = $2")
            .bind(user_id)
            .bind(status)
            .fetch_one(&mut conn)
            .await?;

    let position = max_pos.unwrap_or(0) + 10_000;

    let task = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (user_id, title, description, status, position)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, user_id, title, description, status, time_spent_secs,
                   position, due_at, created_at, updated_at",
    )
    .bind(user_id)
    .bind(&title)
    .bind(&description)
    .bind(status)
    .bind(position)
    .fetch_one(&mut conn)
    .await?;

    Ok(task)
}

#[forge::mutation]
pub async fn update_task(ctx: &MutationContext, input: UpdateTaskInput) -> Result<Task> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let existing = sqlx::query_as::<_, Task>(
        "SELECT id, user_id, title, description, status, time_spent_secs,
                position, due_at, created_at, updated_at
         FROM tasks WHERE id = $1 AND user_id = $2",
    )
    .bind(input.id)
    .bind(user_id)
    .fetch_optional(&mut conn)
    .await?
    .ok_or_else(|| ForgeError::NotFound("Task not found".into()))?;

    let title = input.title.unwrap_or(existing.title);
    let description = input.description.unwrap_or(existing.description);
    let status = input.status.unwrap_or(existing.status);
    let due_at = match input.due_at {
        Some(d) => d,
        None => existing.due_at,
    };

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET title = $1, description = $2, status = $3, due_at = $4,
                updated_at = NOW()
         WHERE id = $5 AND user_id = $6
         RETURNING id, user_id, title, description, status, time_spent_secs,
                   position, due_at, created_at, updated_at",
    )
    .bind(&title)
    .bind(&description)
    .bind(status)
    .bind(due_at)
    .bind(input.id)
    .bind(user_id)
    .fetch_one(&mut conn)
    .await?;

    Ok(task)
}

#[forge::mutation]
pub async fn delete_task(ctx: &MutationContext, input: DeleteTaskInput) -> Result<()> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let rows = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(input.id)
        .bind(user_id)
        .execute(&mut conn)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ForgeError::NotFound("Task not found".into()));
    }

    Ok(())
}

#[forge::mutation]
pub async fn focus_task(ctx: &MutationContext, input: FocusTaskInput) -> Result<Task> {
    let user_id = ctx.user_id()?;
    let now = Utc::now();
    let mut conn = ctx.conn().await?;

    // Defocus the currently focused task, if any
    let focused = sqlx::query_as::<_, Task>(
        "SELECT id, user_id, title, description, status, time_spent_secs,
                position, due_at, created_at, updated_at
         FROM tasks WHERE user_id = $1 AND status = 'focused'",
    )
    .bind(user_id)
    .fetch_optional(&mut conn)
    .await?;

    if let Some(prev) = focused {
        if prev.id == input.id {
            // Already focused, just return it
            return Ok(prev);
        }
        let elapsed = (now - prev.updated_at).num_seconds().max(0);
        let new_time = prev.time_spent_secs + elapsed;

        sqlx::query(
            "UPDATE tasks SET status = 'in_progress', time_spent_secs = $1, updated_at = $2
             WHERE id = $3",
        )
        .bind(new_time)
        .bind(now)
        .bind(prev.id)
        .execute(&mut conn)
        .await?;
    }

    // Focus the new task
    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET status = 'focused', updated_at = $1
         WHERE id = $2 AND user_id = $3
         RETURNING id, user_id, title, description, status, time_spent_secs,
                   position, due_at, created_at, updated_at",
    )
    .bind(now)
    .bind(input.id)
    .bind(user_id)
    .fetch_optional(&mut conn)
    .await?
    .ok_or_else(|| ForgeError::NotFound("Task not found".into()))?;

    Ok(task)
}

#[forge::mutation]
pub async fn unfocus_task(ctx: &MutationContext) -> Result<Task> {
    let user_id = ctx.user_id()?;
    let now = Utc::now();
    let mut conn = ctx.conn().await?;

    let focused = sqlx::query_as::<_, Task>(
        "SELECT id, user_id, title, description, status, time_spent_secs,
                position, due_at, created_at, updated_at
         FROM tasks WHERE user_id = $1 AND status = 'focused'",
    )
    .bind(user_id)
    .fetch_optional(&mut conn)
    .await?
    .ok_or_else(|| ForgeError::NotFound("No focused task".into()))?;

    let elapsed = (now - focused.updated_at).num_seconds().max(0);
    let new_time = focused.time_spent_secs + elapsed;

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET status = 'in_progress', time_spent_secs = $1, updated_at = $2
         WHERE id = $3
         RETURNING id, user_id, title, description, status, time_spent_secs,
                   position, due_at, created_at, updated_at",
    )
    .bind(new_time)
    .bind(now)
    .bind(focused.id)
    .fetch_one(&mut conn)
    .await?;

    Ok(task)
}

#[forge::mutation]
pub async fn reorder_task(ctx: &MutationContext, input: ReorderTaskInput) -> Result<Task> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET status = $1, position = $2, updated_at = NOW()
         WHERE id = $3 AND user_id = $4
         RETURNING id, user_id, title, description, status, time_spent_secs,
                   position, due_at, created_at, updated_at",
    )
    .bind(input.status)
    .bind(input.position)
    .bind(input.id)
    .bind(user_id)
    .fetch_optional(&mut conn)
    .await?
    .ok_or_else(|| ForgeError::NotFound("Task not found".into()))?;

    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge::testing::IsolatedTestDb;
    use std::path::Path;

    async fn setup_db() -> IsolatedTestDb {
        IsolatedTestDb::setup(
            "tasks_test",
            &forge::get_internal_sql(),
            Path::new("migrations"),
        )
        .await
        .unwrap()
    }

    async fn insert_user(pool: &sqlx::PgPool) -> Uuid {
        let hash = bcrypt::hash("password123", 4).unwrap();
        let user = sqlx::query_as::<_, crate::schema::User>(
            "INSERT INTO users (email, name, password_hash)
             VALUES ($1, $2, $3)
             RETURNING id, email, name, password_hash, created_at, updated_at",
        )
        .bind(format!("test-{}@example.com", Uuid::new_v4()))
        .bind("Tester")
        .bind(&hash)
        .fetch_one(pool)
        .await
        .unwrap();
        user.id
    }

    async fn insert_task(pool: &sqlx::PgPool, user_id: Uuid, title: &str) -> Task {
        sqlx::query_as::<_, Task>(
            "INSERT INTO tasks (user_id, title, status, position)
             VALUES ($1, $2, 'inbox', 10000)
             RETURNING id, user_id, title, description, status, time_spent_secs,
                       position, due_at, created_at, updated_at",
        )
        .bind(user_id)
        .bind(title)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn task_defaults_to_inbox() {
        let db = setup_db().await;
        let uid = insert_user(db.pool()).await;
        let task = insert_task(db.pool(), uid, "My task").await;

        assert_eq!(task.title, "My task");
        assert_eq!(task.status, TaskStatus::Inbox);
        assert_eq!(task.user_id, uid);
        assert_eq!(task.time_spent_secs, 0);

        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn tasks_scoped_to_user() {
        let db = setup_db().await;
        let uid_a = insert_user(db.pool()).await;
        let uid_b = insert_user(db.pool()).await;

        insert_task(db.pool(), uid_a, "A's task").await;
        insert_task(db.pool(), uid_b, "B's task").await;

        let tasks_a = sqlx::query_as::<_, Task>(
            "SELECT id, user_id, title, description, status, time_spent_secs,
                    position, due_at, created_at, updated_at
             FROM tasks WHERE user_id = $1",
        )
        .bind(uid_a)
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(tasks_a.len(), 1);
        assert_eq!(tasks_a[0].title, "A's task");

        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn update_task_changes_fields() {
        let db = setup_db().await;
        let uid = insert_user(db.pool()).await;
        let task = insert_task(db.pool(), uid, "Original").await;

        let updated = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET title = $1, description = $2, status = 'up_next'
             WHERE id = $3
             RETURNING id, user_id, title, description, status, time_spent_secs,
                       position, due_at, created_at, updated_at",
        )
        .bind("Updated")
        .bind("A description")
        .bind(task.id)
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.description, "A description");
        assert_eq!(updated.status, TaskStatus::UpNext);

        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn delete_task_removes_it() {
        let db = setup_db().await;
        let uid = insert_user(db.pool()).await;
        let task = insert_task(db.pool(), uid, "To delete").await;

        sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(task.id)
            .execute(db.pool())
            .await
            .unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks WHERE user_id = $1")
            .bind(uid)
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert_eq!(count.0, 0);
        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn only_one_focused_task_per_user() {
        let db = setup_db().await;
        let uid = insert_user(db.pool()).await;

        let t1 = insert_task(db.pool(), uid, "Task 1").await;
        let t2 = insert_task(db.pool(), uid, "Task 2").await;

        // Focus task 1
        sqlx::query("UPDATE tasks SET status = 'focused' WHERE id = $1")
            .bind(t1.id)
            .execute(db.pool())
            .await
            .unwrap();

        // Defocus t1, focus t2 (simulates focus_task logic)
        sqlx::query("UPDATE tasks SET status = 'in_progress' WHERE id = $1")
            .bind(t1.id)
            .execute(db.pool())
            .await
            .unwrap();
        sqlx::query("UPDATE tasks SET status = 'focused' WHERE id = $1")
            .bind(t2.id)
            .execute(db.pool())
            .await
            .unwrap();

        let focused: Vec<Task> = sqlx::query_as(
            "SELECT id, user_id, title, description, status, time_spent_secs,
                    position, due_at, created_at, updated_at
             FROM tasks WHERE user_id = $1 AND status = 'focused'",
        )
        .bind(uid)
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(focused.len(), 1);
        assert_eq!(focused[0].id, t2.id);

        // Verify t1 is in_progress
        let t1_status = sqlx::query_as::<_, Task>(
            "SELECT id, user_id, title, description, status, time_spent_secs,
                    position, due_at, created_at, updated_at
             FROM tasks WHERE id = $1",
        )
        .bind(t1.id)
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(t1_status.status, TaskStatus::InProgress);
        db.cleanup().await.unwrap();
    }
}
