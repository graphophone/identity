use identity::config::Config;

fn main() {
    let conf = Config::build("config/config.local.toml")
        .expect("failed to read config");

    identity::run(conf)
        .expect("failed when running identity service");
}
