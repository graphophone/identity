use crate::database::IdentityDb;

pub mod config;
mod database;

pub async fn run(conf: config::Config) -> anyhow::Result<()> {
    println!("conf: {:?}", conf);

    let identity_db = IdentityDb::build(&conf.postgres)
        .await
        .expect("failed to connect to database");

    Ok(())
}