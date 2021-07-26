// Database table structs and database layer methods
use sqlx::postgres::PgPool;

use crate::auth;
use crate::msg::{UpdateUser, UserJson, UserResponse};

// User table in database
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    hash: String,
}

impl User {
    fn token(&self) -> String {
        let expires = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(60))
            .unwrap()
            .timestamp();

        auth::generate_token(self.id, self.username.clone(), expires).unwrap()
    }

    pub fn to_user_response(self) -> UserResponse {
        let token = self.token();
        UserResponse {
            user: UserJson {
                email: self.email,
                token,
                username: self.username,
                bio: self.bio,
                image: self.image,
            },
        }
    }

    pub fn compare_password(&self, plain_password: &str) -> bool {
        auth::verify_password(plain_password, &self.hash)
    }
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var missing.");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Error connecting to database");

        tide::log::info!("Database pool created");
        Database { pool }
    }

    pub async fn register_user(
        &self,
        email: String,
        username: String,
        hashed_password: String,
    ) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
INSERT INTO users (email, username, hash)
VALUES ( $1, $2, $3)
RETURNING *
            "#,
            email,
            username,
            hashed_password
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
SELECT *
FROM users
WHERE email = $1
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: i32) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
SELECT *
FROM users
WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user(
        &self,
        id: i32,
        update_user: UpdateUser,
        hash: Option<String>,
    ) -> anyhow::Result<User> {
        // update using coalesce to prevent overwriting data with null
        // is there a better way?
        let user = sqlx::query_as!(
            User,
            r#"
UPDATE users
SET
    email = COALESCE($2, email),
    username = COALESCE($3, username),
    hash = COALESCE($4, hash),
    image = COALESCE($5, image),
    bio = COALESCE($6, bio)
WHERE id = $1
RETURNING *
            "#,
            id,
            update_user.email,
            update_user.username,
            hash,
            update_user.image,
            update_user.bio
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn contains_email(&self, email: &str) -> anyhow::Result<bool> {
        let record_option = sqlx::query!(
            r#"
SELECT *
FROM users
WHERE email = $1
            "#,
            email,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record_option.is_some())
    }

    pub async fn contains_username(&self, username: &str) -> anyhow::Result<bool> {
        let record_option = sqlx::query!(
            r#"
SELECT *
FROM users
WHERE username = $1
            "#,
            username,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record_option.is_some())
    }
}
