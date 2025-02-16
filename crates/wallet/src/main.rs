use clap::Parser as _;

mod cli;
mod keygen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = x_link_utils::logging::init_logger();
    cli::Args::parse().run()
}
