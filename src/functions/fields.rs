use forge::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    schema::{FIELD_DEFINITION_COLUMNS, FieldDefinition, FieldValueType},
    support::{next_position, required_trimmed},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFieldDefinitionInput {
    pub key: String,
    pub value_type: FieldValueType,
    pub color: Option<String>,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFieldDefinitionInput {
    pub id: Uuid,
    pub key: Option<String>,
    pub color: Option<String>,
    pub options: Option<Vec<String>>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFieldDefinitionInput {
    pub id: Uuid,
}

fn normalize_field_key(key: &str) -> Result<String> {
    Ok(required_trimmed(key, "Key is required")?.to_lowercase())
}

#[forge::query]
pub async fn list_field_definitions(ctx: &QueryContext) -> Result<Vec<FieldDefinition>> {
    let user_id = ctx.user_id()?;
    let query = format!(
        "SELECT {FIELD_DEFINITION_COLUMNS} FROM field_definitions WHERE user_id = $1 ORDER BY position ASC, created_at ASC"
    );

    let fields = sqlx::query_as::<_, FieldDefinition>(&query)
        .bind(user_id)
        .fetch_all(ctx.db())
        .await?;
    Ok(fields)
}

#[forge::mutation]
pub async fn create_field_definition(
    ctx: &MutationContext,
    input: CreateFieldDefinitionInput,
) -> Result<FieldDefinition> {
    let key = normalize_field_key(&input.key)?;
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let max_pos: Option<i32> =
        sqlx::query_scalar("SELECT MAX(position) FROM field_definitions WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&mut conn)
            .await?;

    let position = next_position(max_pos);
    let insert_field_query = format!(
        "INSERT INTO field_definitions (user_id, key, value_type, color, options, position) VALUES ($1, $2, $3, $4, $5, $6) RETURNING {FIELD_DEFINITION_COLUMNS}"
    );

    let field = sqlx::query_as::<_, FieldDefinition>(&insert_field_query)
        .bind(user_id)
        .bind(&key)
        .bind(input.value_type)
        .bind(input.color.as_deref())
        .bind(&input.options)
        .bind(position)
        .fetch_one(&mut conn)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ForgeError::Validation(format!("Field '{key}' already exists"))
            }
            other => ForgeError::Sql(other),
        })?;

    Ok(field)
}

#[forge::mutation]
pub async fn update_field_definition(
    ctx: &MutationContext,
    input: UpdateFieldDefinitionInput,
) -> Result<FieldDefinition> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let select_field_query = format!(
        "SELECT {FIELD_DEFINITION_COLUMNS} FROM field_definitions WHERE id = $1 AND user_id = $2"
    );

    let existing = sqlx::query_as::<_, FieldDefinition>(&select_field_query)
        .bind(input.id)
        .bind(user_id)
        .fetch_optional(&mut conn)
        .await?
        .ok_or_else(|| ForgeError::NotFound("Field definition not found".into()))?;

    let key = match input.key {
        Some(key) => normalize_field_key(&key)?,
        None => existing.key,
    };
    let color = input.color.or(existing.color);
    let options = input.options.or(existing.options);
    let position = input.position.unwrap_or(existing.position);

    let update_field_query = format!(
        "UPDATE field_definitions SET key = $1, color = $2, options = $3, position = $4 WHERE id = $5 AND user_id = $6 RETURNING {FIELD_DEFINITION_COLUMNS}"
    );

    let field = sqlx::query_as::<_, FieldDefinition>(&update_field_query)
        .bind(&key)
        .bind(color.as_deref())
        .bind(&options)
        .bind(position)
        .bind(input.id)
        .bind(user_id)
        .fetch_one(&mut conn)
        .await?;

    Ok(field)
}

#[forge::mutation]
pub async fn delete_field_definition(
    ctx: &MutationContext,
    input: DeleteFieldDefinitionInput,
) -> Result<()> {
    let user_id = ctx.user_id()?;
    let mut conn = ctx.conn().await?;

    let rows = sqlx::query("DELETE FROM field_definitions WHERE id = $1 AND user_id = $2")
        .bind(input.id)
        .bind(user_id)
        .execute(&mut conn)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ForgeError::NotFound("Field definition not found".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::FieldDefinition;
    use crate::test_support::{insert_unique_test_user, require_test_db, setup_test_db};

    #[tokio::test]
    async fn create_and_list_field_definitions() {
        if !require_test_db() {
            return;
        }
        let db = setup_test_db("fields_test").await;
        let uid = insert_unique_test_user(db.pool()).await;

        sqlx::query(
            "INSERT INTO field_definitions (user_id, key, value_type, position)
             VALUES ($1, 'priority', 'enum', 10000)",
        )
        .bind(uid)
        .execute(db.pool())
        .await
        .unwrap();

        let fields = sqlx::query_as::<_, FieldDefinition>(
            "SELECT id, user_id, key, value_type, color, options, position, created_at
             FROM field_definitions WHERE user_id = $1",
        )
        .bind(uid)
        .fetch_all(db.pool())
        .await
        .unwrap();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "priority");
        assert_eq!(fields[0].value_type, FieldValueType::Enum);

        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn duplicate_key_rejected() {
        if !require_test_db() {
            return;
        }
        let db = setup_test_db("fields_test").await;
        let uid = insert_unique_test_user(db.pool()).await;

        sqlx::query(
            "INSERT INTO field_definitions (user_id, key, value_type, position)
             VALUES ($1, 'priority', 'enum', 10000)",
        )
        .bind(uid)
        .execute(db.pool())
        .await
        .unwrap();

        let result = sqlx::query(
            "INSERT INTO field_definitions (user_id, key, value_type, position)
             VALUES ($1, 'priority', 'text', 20000)",
        )
        .bind(uid)
        .execute(db.pool())
        .await;

        assert!(result.is_err());
        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn cascade_delete_removes_task_fields() {
        if !require_test_db() {
            return;
        }
        let db = setup_test_db("fields_test").await;
        let uid = insert_unique_test_user(db.pool()).await;

        let field_id: Uuid = sqlx::query_scalar(
            "INSERT INTO field_definitions (user_id, key, value_type, position)
             VALUES ($1, 'priority', 'enum', 10000) RETURNING id",
        )
        .bind(uid)
        .fetch_one(db.pool())
        .await
        .unwrap();

        let task_id: Uuid = sqlx::query_scalar(
            "INSERT INTO tasks (user_id, title, position)
             VALUES ($1, 'Test task', 10000) RETURNING id",
        )
        .bind(uid)
        .fetch_one(db.pool())
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO task_fields (task_id, field_id, value)
             VALUES ($1, $2, 'high')",
        )
        .bind(task_id)
        .bind(field_id)
        .execute(db.pool())
        .await
        .unwrap();

        // Delete the field definition
        sqlx::query("DELETE FROM field_definitions WHERE id = $1")
            .bind(field_id)
            .execute(db.pool())
            .await
            .unwrap();

        // Task fields should be gone
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM task_fields WHERE field_id = $1")
            .bind(field_id)
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert_eq!(count.0, 0);
        db.cleanup().await.unwrap();
    }
}
