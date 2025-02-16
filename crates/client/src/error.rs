#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("RPC error: {0}")]
    Generic(String),

    #[error("X-Link Solana error: {0}")]
    Client(#[from] x_link_solana::error::Error),
}
