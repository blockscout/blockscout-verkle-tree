use verkle_tree_img::{run, Args, Config};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::default();
    let config = Config::from_file(args.config_path).expect("Failed to parse config");
    run(config).await
}
