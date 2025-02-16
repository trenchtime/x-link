use clap::Parser as _;
use x_link_client::cli::Args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = x_link_utils::logging::init_logger();
    let args = Args::parse();
    args.run().await.map_err(Into::into)
}
