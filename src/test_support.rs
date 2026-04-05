use forge::testing::IsolatedTestDb;
use std::path::Path;
use uuid::Uuid;

use crate::schema::User;

pub fn require_test_db() -> bool {
    dotenvy::dotenv().ok();
    std::env::var("TEST_DATABASE_URL").is_ok()
}

pub async fn setup_test_db(name: &str) -> IsolatedTestDb {
    IsolatedTestDb::setup(name, &forge::get_internal_sql(), Path::new("migrations"))
        .await
        .unwrap()
}

pub async fn insert_test_user(pool: &sqlx::PgPool, email: &str, password: &str) -> User {
    let hash = bcrypt::hash(password, 4).unwrap();

    sqlx::query_as::<_, User>(
        "INSERT INTO users (email, name, password_hash)
         VALUES ($1, $2, $3)
         RETURNING id, email, name, password_hash, created_at, updated_at",
    )
    .bind(email)
    .bind("Test User")
    .bind(&hash)
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn insert_unique_test_user(pool: &sqlx::PgPool) -> Uuid {
    insert_test_user(
        pool,
        &format!("test-{}@example.com", Uuid::new_v4()),
        "password123",
    )
    .await
    .id
}
