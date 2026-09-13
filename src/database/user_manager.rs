use sqlx::{PgConnection, Result};

use crate::{database::IdentityDb, util};

pub trait UserManager {
    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<i64>;
    async fn delete_user(&self, user_id: i64) -> Result<()>;
    async fn update_avatar(&self, user_id: i64, key: &str) -> Result<Option<String>>;
    async fn update_username(&self, user_id: i64, username: &str) -> Result<()>;
    async fn update_password(&self, user_id: i64, old_password: &str, new_password: &str) -> anyhow::Result<()>;
    async fn verify_password(tx: &mut PgConnection, user_id: i64, password: &str) -> anyhow::Result<bool>;
}

impl UserManager for IdentityDb {
    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<i64> {
        let password_hash = util::password::hash_password(password)?;
        let id = sqlx::query_scalar!(r"
            INSERT INTO users (username, password_hash)
            VALUES ($1, $2)
            RETURNING id;
        ", username, password_hash)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }
    
    async fn delete_user(&self, user_id: i64) -> Result<()> {
        sqlx::query!(r"
            DELETE FROM users WHERE id = $1;
        ", user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn update_username(&self, user_id: i64, username: &str) -> Result<()> {
        sqlx::query!(r"
            UPDATE users SET username = $1
            WHERE id = $2
        ", username, user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn update_password(&self, user_id: i64, old_password: &str, new_password: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        match Self::verify_password(&mut tx, user_id, old_password).await {
            Ok(false) => return Err(anyhow::Error::msg("passwords do not match")),
            Err(e) => return Err(e),
            _ => (),
        };

        let password_hash = util::password::hash_password(new_password)?;
        sqlx::query!(r"
            UPDATE users
            SET password_hash = $2
            WHERE id = $1
        ", user_id, password_hash)
            .execute(tx.as_mut())
            .await?;

        tx.commit().await?;
        Ok(())
    }
    
    async fn verify_password(tx: &mut PgConnection, user_id: i64, password: &str) -> anyhow::Result<bool> {
        let password_hash = sqlx::query_scalar!(r"
            SELECT password_hash
            FROM users
            WHERE id = $1;
        ", user_id)
            .fetch_one(tx)
            .await?;

        util::password::verify_password(password, &password_hash)
            .map_err(anyhow::Error::from)
    }
    
    async fn update_avatar(&self, user_id: i64, url: &str) -> Result<Option<String>> {
        let old_url: Option<String> = sqlx::query_scalar!(r"
            UPDATE users
            SET avatar_url = $2
            WHERE id = $1
            RETURNING OLD.avatar_url
        ", user_id, url)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(old_url)
    }
}