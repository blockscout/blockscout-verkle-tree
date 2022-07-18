
#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    log::info!("hello, world!");
    Ok(())
}