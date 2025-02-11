use clap::Parser as _;

mod cli;
mod keygen;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = utils::init_logger();
    cli::Args::parse().run()
}
