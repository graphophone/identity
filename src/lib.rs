pub mod config;

pub fn run(conf: config::Config) -> anyhow::Result<()> {
    println!("conf: {:?}", conf);

    Ok(())
}