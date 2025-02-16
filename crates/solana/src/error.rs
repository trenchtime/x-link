#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("RPC error: {0}")]
    Generic(String),

    #[error("Client error: {0}")]
    Client(#[from] jupiter_swap_api_client::ClientError),

    #[error("Solana client error: {0}")]
    SolanaClient(#[from] solana_client::client_error::ClientError),
}
