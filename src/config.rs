use std::fs;

use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Config {
    pub postgres: PostgresConfig,
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