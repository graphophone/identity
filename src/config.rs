use std::fs;

use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Config {
    pub postgres: PostgresConfig,
    pub rustfs: RustfsConfig,
}

impl Config {
    pub fn build(filename: &str) -> anyhow::Result<Self> {
        let conf_content = fs::read_to_string(filename)?;
        toml::from_str(&conf_content)
            .map_err(anyhow::Error::from)
    }
}

#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: i32,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct RustfsConfig {
    pub access_key: String,
    pub secret_key: String,
    pub endpoint_url: String,
    pub region: String,
    pub assets_bucket: String,
}