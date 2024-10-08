use chrono::NaiveDateTime;
use data_store::{generate_push_binds, GetFieldNames};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, QueryBuilder};

const BIND_LIMIT: usize = 65535;

#[derive(Debug, Default, Serialize, Deserialize, FromRow, GetFieldNames)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

async fn add_user(pool: &PgPool, user: User) -> anyhow::Result<i32> {
    let rec = sqlx::query!(
        r#"
INSERT INTO users (username, email)
VALUES ($1, $2)
RETURNING id
        "#,
        user.username,
        user.email
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}

async fn insert_users_build(
    pool: &PgPool,
    users: impl Iterator<Item = User>,
) -> Result<(), sqlx::Error> {
    let field_names: Vec<&str> = User::field_names()
        .iter()
        .filter(|&&field| field != "id")
        .copied()
        .collect();

    let mut query_builder =
        QueryBuilder::new(format!("INSERT INTO users ({}) ", field_names.join(", ")));

    query_builder.push_values(users.take(BIND_LIMIT / 4), |mut b, user| {
        // generate_push_binds!(b, user, [username, email, created_at]);
        generate_push_binds!([b, user, username, email, created_at]);
    });

    let query = query_builder.build();
    let _result = query.execute(pool).await?;

    Ok(())
}

#[tokio::test]
async fn test_add_user() -> anyhow::Result<()> {
    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    let user = User {
        username: "Licke".to_string(),
        email: "test2".to_string(),
        ..Default::default()
    };
    add_user(&pool, user).await?;
    Ok(())
}

#[tokio::test]
async fn test_add_users() -> anyhow::Result<()> {
    let pool = PgPool::connect(&dotenvy::var("DATABASE_URL")?).await?;
    let users = (0..10).map(|i| User {
        username: format!("test_user_{i}"),
        email: format!("test-user-{i}@example.com"),
        created_at: NaiveDateTime::parse_from_str("2024-09-19 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap_or_default(),
        ..Default::default()
    });

    insert_users_build(&pool, users).await?;
    Ok(())
}
