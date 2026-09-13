use sqlx::Result;

use crate::{database::IdentityDb, util};

pub trait UserManager {
    async fn get_profiles(&self, user_ids: &[i64]) -> Result<Vec<Profile>>;
    async fn get_full_profile(&self, user_id: i64) -> Result<FullProfile>;
    async fn create_user(&self, username: &str, email: &str, password: &str) -> anyhow::Result<i64>;
    async fn delete_user(&self, user_id: i64) -> Result<()>;
    async fn update_profile(&self, user_id: i64, metadata: &ProfileMetadata) -> Result<()>;
    async fn update_avatar(&self, user_id: i64, key: &str) -> Result<Option<String>>;
    async fn update_password(&self, user_id: i64, old_password: &str, new_password: &str) -> anyhow::Result<()>;
    async fn verify_password(&self, user_id: i64, password: &str) -> anyhow::Result<bool>;
}

impl UserManager for IdentityDb {
    async fn create_user(&self, username: &str, email: &str, password: &str) -> anyhow::Result<i64> {
        let password_hash = util::password::hash_password(password)?;
        let id = sqlx::query_scalar!(r"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id;
        ", username, email, password_hash)
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
    
    async fn update_password(&self, user_id: i64, old_password: &str, new_password: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let password_hash = sqlx::query_scalar!(r"
            SELECT password_hash
            FROM users
            WHERE id = $1;
        ", user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(anyhow::Error::from)?;

        let is_valid = util::password::verify_password(old_password, &password_hash)
            .map_err(anyhow::Error::from)?;
        if !is_valid {
            return Err(anyhow::Error::msg("passwords do not match"));
        }

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
    
    async fn update_avatar(&self, user_id: i64, url: &str) -> Result<Option<String>> {
        let old_url: Option<String> = sqlx::query_scalar!(r"
            UPDATE users
            SET avatar_url = $2
            WHERE id = $1
            RETURNING OLD.avatar_url;
        ", user_id, url)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(old_url)
    }
    
    async fn get_profiles(&self, user_ids: &[i64]) -> Result<Vec<Profile>> {
        let profiles: Vec<Profile> = sqlx::query_as!(Profile, r"
            SELECT id user_id, username, avatar_url
            FROM users
            WHERE id IN (SELECT * FROM UNNEST($1::bigint[]));
        ", user_ids)
            .fetch_all(&self.pool)
            .await?;

        Ok(profiles)
    }
    
    async fn get_full_profile(&self, user_id: i64) -> Result<FullProfile> {
        let profile = sqlx::query_as!(FullProfile, r"
            SELECT
                id user_id,
                username,
                avatar_url,
                first_name,
                last_name,
                bio,
                country,
                city
            FROM users
            WHERE id = $1;
        ", user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(profile)
    }
    
    async fn update_profile(&self, user_id: i64, metadata: &ProfileMetadata) -> Result<()> {
        sqlx::query!(r"
            UPDATE users
            SET 
                username = $2,
                first_name = $3,
                last_name = $4,
                bio = $5,
                email = $6,
                country = $7,
                city = $8
            WHERE id = $1
            ",
            user_id,
            metadata.username,
            metadata.first_name,
            metadata.last_name,
            metadata.bio,
            metadata.email,
            metadata.country,
            metadata.city,
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    
    async fn verify_password(&self, user_id: i64, password: &str) -> anyhow::Result<bool> {
        let password_hash = sqlx::query_scalar!(r"
            SELECT password_hash
            FROM users
            WHERE id = $1;
        ", user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(anyhow::Error::from)?;

        util::password::verify_password(password, &password_hash)
            .map_err(anyhow::Error::from)
    }
}

pub struct Profile {
    pub user_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
}

pub struct FullProfile {
    pub user_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}

pub struct ProfileMetadata {
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}