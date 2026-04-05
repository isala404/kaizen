use forge::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    schema::{USER_COLUMNS, User, Viewer},
    support::required_trimmed,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshInput {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutInput {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub viewer: Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

fn normalize_email(email: &str) -> Result<String> {
    Ok(required_trimmed(email, "Email is required")?.to_lowercase())
}

fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(ForgeError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }

    Ok(())
}

async fn find_user_by_email<'a, E>(executor: E, email: &str) -> Result<Option<User>>
where
    E: sqlx::PgExecutor<'a>,
{
    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE email = $1");

    let user = sqlx::query_as::<_, User>(&query)
        .bind(email)
        .fetch_optional(executor)
        .await?;

    Ok(user)
}

async fn find_user_by_id<'a, E>(executor: E, user_id: uuid::Uuid) -> Result<Option<User>>
where
    E: sqlx::PgExecutor<'a>,
{
    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");

    let user = sqlx::query_as::<_, User>(&query)
        .bind(user_id)
        .fetch_optional(executor)
        .await?;

    Ok(user)
}

#[forge::mutation(public)]
pub async fn register(ctx: &MutationContext, input: RegisterInput) -> Result<AuthResponse> {
    let name = required_trimmed(&input.name, "Name is required")?;
    let email = normalize_email(&input.email)?;
    validate_password(&input.password)?;

    let hash = bcrypt::hash(&input.password, bcrypt::DEFAULT_COST)
        .map_err(|e| ForgeError::Internal(format!("Failed to hash password: {e}")))?;

    let mut conn = ctx.conn().await?;
    let insert_user_query = format!(
        "INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3) RETURNING {USER_COLUMNS}"
    );

    let user = sqlx::query_as::<_, User>(&insert_user_query)
        .bind(&email)
        .bind(&name)
        .bind(&hash)
        .fetch_one(&mut conn)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ForgeError::Validation("An account with this email already exists".into())
            }
            other => ForgeError::Sql(other),
        })?;

    let pair = ctx.issue_token_pair(user.id, &["user"]).await?;
    let viewer: Viewer = user.into();

    Ok(AuthResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        viewer,
    })
}

#[forge::mutation(public)]
pub async fn login(ctx: &MutationContext, input: LoginInput) -> Result<AuthResponse> {
    let email = normalize_email(&input.email)?;

    let mut conn = ctx.conn().await?;
    let user = find_user_by_email(&mut *conn, &email)
        .await?
        .ok_or_else(|| ForgeError::Unauthorized("Invalid email or password".into()))?;

    let valid = bcrypt::verify(&input.password, &user.password_hash)
        .map_err(|e| ForgeError::Internal(format!("Failed to verify password: {e}")))?;

    if !valid {
        return Err(ForgeError::Unauthorized("Invalid email or password".into()));
    }

    let pair = ctx.issue_token_pair(user.id, &["user"]).await?;
    let viewer: Viewer = user.into();

    Ok(AuthResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        viewer,
    })
}

#[forge::mutation(public)]
pub async fn refresh(ctx: &MutationContext, input: RefreshInput) -> Result<RefreshResponse> {
    let pair = ctx.rotate_refresh_token(&input.refresh_token).await?;
    Ok(RefreshResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
    })
}

#[forge::mutation]
pub async fn logout(ctx: &MutationContext, input: LogoutInput) -> Result<()> {
    ctx.revoke_refresh_token(&input.refresh_token).await
}

#[forge::query(unscoped)]
pub async fn get_me(ctx: &QueryContext) -> Result<Viewer> {
    let user_id = ctx.user_id()?;
    let user = find_user_by_id(ctx.db(), user_id)
        .await?
        .ok_or_else(|| ForgeError::NotFound("User not found".into()))?;

    Ok(user.into())
}

#[cfg(test)]
mod tests {
    use crate::schema::User;
    use crate::test_support::{insert_test_user, require_test_db, setup_test_db};

    #[tokio::test]
    async fn bcrypt_hash_and_verify() {
        let hash = bcrypt::hash("password123", 4).unwrap();
        assert!(bcrypt::verify("password123", &hash).unwrap());
        assert!(!bcrypt::verify("wrongpassword", &hash).unwrap());
    }

    #[tokio::test]
    async fn insert_user_creates_record() {
        if !require_test_db() {
            return;
        }
        let db = setup_test_db("auth_test").await;
        let user = insert_test_user(db.pool(), "test@example.com", "password123").await;

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert!(!user.password_hash.is_empty());

        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn duplicate_email_rejected() {
        if !require_test_db() {
            return;
        }
        let db = setup_test_db("auth_test").await;
        insert_test_user(db.pool(), "dupe@example.com", "password123").await;

        let result =
            sqlx::query("INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3)")
                .bind("dupe@example.com")
                .bind("Another")
                .bind("hash")
                .execute(db.pool())
                .await;

        assert!(result.is_err());
        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn user_lookup_by_email() {
        if !require_test_db() {
            return;
        }
        let db = setup_test_db("auth_test").await;
        insert_test_user(db.pool(), "find@example.com", "password123").await;

        let found = sqlx::query_as::<_, User>(
            "SELECT id, email, name, password_hash, created_at, updated_at
             FROM users WHERE email = $1",
        )
        .bind("find@example.com")
        .fetch_optional(db.pool())
        .await
        .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "find@example.com");

        let not_found = sqlx::query_as::<_, User>(
            "SELECT id, email, name, password_hash, created_at, updated_at
             FROM users WHERE email = $1",
        )
        .bind("missing@example.com")
        .fetch_optional(db.pool())
        .await
        .unwrap();

        assert!(not_found.is_none());
        db.cleanup().await.unwrap();
    }
}
