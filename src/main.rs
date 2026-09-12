use identity::config::Config;

#[tokio::main]
async fn main() {
    let conf = Config::build("config/config.local.toml")
        .expect("failed to read config");

    identity::run(conf)
        .await
        .expect("failed when running identity service");
}
