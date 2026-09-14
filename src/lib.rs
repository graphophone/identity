use tonic::transport::Server;
use tower_http::trace::TraceLayer;

use crate::{avatar_storage::AvatarStorage, database::IdentityDb, service::{IdentityService, identity::identity_server::IdentityServer}};

pub mod config;
mod database;
mod util;
mod avatar_storage;
mod service;

pub async fn run(conf: config::Config) -> anyhow::Result<()> {
    let addr = "0.0.0.0:8080".parse()?;

    let identity_db = IdentityDb::build(&conf.postgres)
        .await
        .expect("failed to connect to database");
    let avatar_storage = AvatarStorage::build(&conf.rustfs)
        .await
        .expect("failed to connect to avatar storage");
    let identity_service = IdentityService::new(identity_db, avatar_storage);

    println!("starting identity service");
    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(IdentityServer::new(identity_service))
        .serve(addr)
        .await?;

    Ok(())
}