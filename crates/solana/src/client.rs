use std::sync::Arc;

use crate::{
    backend::jupiter,
    constants::{HASH_EXPIRATION, NATIVE_MINT, SOL_BASE_PATH},
    error::Error,
    fresh_hash::FreshHash,
};
use dashmap::DashMap;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};
use x_link_types::account::Account;

use solana_sdk::hash::Hash;

pub struct CachedHash {
    hash: Hash,
    time: std::time::Instant,
}

impl CachedHash {
    const EXPIRATION: std::time::Duration = std::time::Duration::from_secs(15);
    pub fn new(hash: Hash) -> Self {
        Self {
            hash,
            time: std::time::Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.time.elapsed() > Self::EXPIRATION
    }

    pub async fn refresh(&mut self, client: &RpcClient) -> Result<(), Error> {
        self.hash = client.get_latest_blockhash().await?;
        self.time = std::time::Instant::now();
        Ok(())
    }

    pub async fn try_get(&self) -> Option<Hash> {
        if !self.is_expired() {
            Some(self.hash)
        } else {
            None
        }
    }
}

pub struct Client {
    jup: jupiter::Backend,
    sol: Arc<RpcClient>,
    trench_tokens: DashMap<Pubkey, bool>,
    fresh_hash: FreshHash,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            jup: jupiter::Backend::new(),
            sol: Arc::new(RpcClient::new(SOL_BASE_PATH.to_string())),
            fresh_hash: FreshHash::new(
                HASH_EXPIRATION,
                Arc::new(RpcClient::new(SOL_BASE_PATH.to_string())),
            ),
            trench_tokens: DashMap::new(),
        }
    }
}

// NOTE: So many fucking options in QuoteRequest and TransactionConfig
// Gotta explore those
impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn recent_blockhash(&self) -> Result<Hash, Error> {
        self.fresh_hash.get().await
    }

    pub async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature, Error> {
        let signature = self.sol.send_transaction(transaction).await?;
        Ok(signature)
    }

    /// SELL `amount` of `mint` for native token
    pub async fn sell(
        &self,
        account: &Account,
        mint: &Pubkey,
        amount: u64,
    ) -> Result<Signature, Error> {
        let recent_blockhash = self.recent_blockhash().await?;
        self.send_transaction(
            &self
                .jup
                .swap_transaction(account, mint, &NATIVE_MINT, amount, recent_blockhash)
                .await?,
        )
        .await
    }

    /// BUY `amount` of `mint` with native token
    pub async fn buy(
        &self,
        account: &Account,
        mint: &Pubkey,
        amount: u64,
    ) -> Result<Signature, Error> {
        let recent_blockhash = self.recent_blockhash().await?;
        self.send_transaction(
            &self
                .jup
                .swap_transaction(account, &NATIVE_MINT, mint, amount, recent_blockhash)
                .await?,
        )
        .await
    }
}
