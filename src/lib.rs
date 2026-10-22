use std::sync::Arc;

use tonic::transport::Server;
use tower_http::trace::TraceLayer;

use crate::{
    asset_storage::AssetStorage,
    database::IdentityDb,
    service::{IdentityService, identity::identity_server::IdentityServer},
};

mod asset_storage;
pub mod config;
mod database;
mod service;
mod util;

pub async fn run(conf: config::Config) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = "0.0.0.0:8080".parse()?;

    let identity_db = IdentityDb::build(&conf.postgres)
        .await
        .expect("failed to connect to database");
    let asset_storage = AssetStorage::build(conf.rustfs.assets_bucket.clone(), &conf.rustfs)
        .await
        .expect("failed to connect to avatar storage");
    let asset_storage = Arc::new(asset_storage);
    let identity_service = IdentityService::new(identity_db, asset_storage);

    println!("starting identity service");
    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(IdentityServer::new(identity_service))
        .serve(addr)
        .await?;

    Ok(())
}
