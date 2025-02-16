use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;

use crate::error::Error;

type HashSender = tokio::sync::mpsc::Sender<tokio::sync::oneshot::Sender<Hash>>;

#[derive(Clone)]
pub struct FreshHash {
    tx: HashSender,
}

impl FreshHash {
    pub fn new(expiration: std::time::Duration, client: Arc<RpcClient>) -> FreshHash {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(FreshHashInner::start(expiration, rx, client.clone()));
        FreshHash { tx }
    }

    pub async fn get(&self) -> Result<Hash, Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(tx).await.map_err(|e| Error::Generic(e.to_string()))?;
        rx.await.map_err(|e| Error::Generic(e.to_string()))
    }
}

pub struct FreshHashInner {
    pub hash: Hash,
    pub expiration: std::time::Duration,
    pub client: Arc<RpcClient>,
}

impl FreshHashInner {
    pub async fn start(
        expiration: std::time::Duration,
        mut rx: tokio::sync::mpsc::Receiver<tokio::sync::oneshot::Sender<Hash>>,
        client: Arc<RpcClient>,
    ) {
        let mut inner = Self {
            hash: Hash::default(),
            expiration,
            client,
        };

        let mut interval = tokio::time::interval(expiration);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match inner.client.get_latest_blockhash().await {
                        Ok(hash) => {
                            inner.hash = hash;
                        }
                        Err(err) => {
                            tracing::error!("Failed to refresh blockhash: {:?}", err);
                        }
                    }
                }
                Some(tx) = rx.recv() => {
                    if let Err(err) = tx.send(inner.hash) {
                        tracing::error!("Failed to send blockhash: {:?}", err);
                    }
                }
            }
        }
    }
}
