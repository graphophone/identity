use std::time::Duration;
use sqlx::{Connection, Pool, Postgres, postgres::PgPoolOptions};
use crate::config::PostgresConfig;

pub mod user_manager;

pub struct IdentityDb {
    pool: Pool<Postgres>,
}

impl IdentityDb {
    pub async fn build(conf: &PostgresConfig) -> anyhow::Result<Self> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            conf.user,
            conf.password,
            conf.host,
            conf.port,
            conf.database,
        );
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(10))
            .connect(&url)
            .await?;

        let mut con = pool.acquire().await?;
        con.ping().await?;
        con.close().await?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(IdentityDb { pool })
    }
}