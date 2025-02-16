use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::body::{Buf, Bytes};
use hyper::Response;
use solana_sdk::signature::Signature;
use x_link_types::account::Account;
use x_link_wallet::keygen::{KeyGen, KeyGenerator as _};

use crate::error::Error;

use crate::message::{
    BuyParams, CreateParams, GetAccountParams, QuoteParams, RpcParams, RpcRequest, RpcResponse,
    SellParams,
};

#[derive(Clone)]
pub struct RpcClient {
    keygen: Arc<KeyGen>,
    backend: Arc<x_link_solana::client::Client>,
}

impl RpcClient {
    pub fn new(keygen: Arc<KeyGen>) -> Self {
        Self {
            keygen,
            backend: Arc::new(x_link_solana::client::Client::default()),
        }
    }

    pub async fn start(secret_file: &str, port: u16) -> Result<(), Error> {
        let passphrase = rpassword::prompt_password("Enter passphrase: ")
            .map_err(|e| Error::Generic(format!("error reading passphrase: {}", e.to_string())))?;
        let keygen = KeyGen::load(secret_file, &passphrase)
            .map_err(|e| Error::Generic(format!("error loading keygen: {}", e.to_string())))?;
        let client = Self::new(Arc::new(keygen));
        client
            .run(port)
            .await
            .map_err(|e| Error::Generic(format!("error running rpc server: {}", e.to_string())))
    }

    pub async fn run(self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("starting rpc server on port: {}", port);
        let listener = tokio::net::TcpListener::bind(self.addr(port)).await?;

        loop {
            let service = self.clone();
            let (stream, _) = listener.accept().await?;
            tracing::debug!("accepted connection from: {:?}", stream.peer_addr());
            let io = hyper_util::rt::TokioIo::new(stream);
            tokio::spawn(async move {
                if let Err(e) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, service)
                    .await
                {
                    tracing::error!("error serving connection: {:?}", e);
                }
            });
        }
    }

    fn addr(&self, port: u16) -> std::net::SocketAddr {
        std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            port,
        )
    }

    fn get_account_by_id(&self, twitter_id: u64) -> Result<Account, Error> {
        Ok(Account {
            twitter_id,
            wallet: self
                .keygen
                .generate_key(twitter_id)
                .map_err(|e| Error::Generic(format!("error generating key: {}", e.to_string())))?,
        })
    }

    async fn handle_buy_inner(
        &self,
        account: Account,
        params: BuyParams,
    ) -> Result<Signature, Error> {
        Ok(self
            .backend
            .buy(&account, &params.token_id, params.amount)
            .await?)
    }

    async fn handle_sell_inner(
        &self,
        account: Account,
        params: SellParams,
    ) -> Result<Signature, Error> {
        Ok(self
            .backend
            .sell(&account, &params.token_id, params.amount)
            .await?)
    }

    async fn handle_buy(&self, id: u64, params: BuyParams) -> RpcResponse {
        match self.get_account_by_id(params.twitter_id) {
            Ok(account) => match self.handle_buy_inner(account, params).await {
                Ok(signature) => RpcResponse::ok(id).with_signature(signature),
                Err(e) => RpcResponse::error(id, &e.to_string()),
            },
            Err(e) => RpcResponse::error(id, &e.to_string()),
        }
    }

    async fn handle_sell(&self, id: u64, params: SellParams) -> RpcResponse {
        match self.get_account_by_id(params.twitter_id) {
            Ok(account) => match self.handle_sell_inner(account, params).await {
                Ok(signature) => RpcResponse::ok(id).with_signature(signature),
                Err(e) => RpcResponse::error(id, &e.to_string()),
            },
            Err(e) => RpcResponse::error(id, &e.to_string()),
        }
    }

    async fn handle_quote(&self, id: u64, params: QuoteParams) -> RpcResponse {
        match self
            .backend
            .quote(params.input_mint, params.output_mint, params.amount)
            .await
        {
            Ok(quote) => RpcResponse::ok(id).with_quote(quote),
            Err(e) => RpcResponse::error(id, &e.to_string()),
        }
    }

    fn handle_create(&self, id: u64, params: CreateParams) -> RpcResponse {
        todo!()
    }

    fn handle_get_account(&self, id: u64, params: GetAccountParams) -> RpcResponse {
        match self.get_account_by_id(params.twitter_id) {
            Ok(account) => RpcResponse::ok(id).with_account(account),
            Err(e) => RpcResponse::error(id, &e.to_string()),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn handle(&self, req: RpcRequest) -> RpcResponse {
        tracing::debug!("handling request");
        match req.params {
            RpcParams::Buy(params) => self.handle_buy(req.id, params).await,
            RpcParams::Sell(params) => self.handle_sell(req.id, params).await,
            RpcParams::Create(params) => self.handle_create(req.id, params),
            RpcParams::GetAccount(params) => self.handle_get_account(req.id, params),
            RpcParams::Quote(params) => self.handle_quote(req.id, params).await,
        }
    }
}

impl hyper::service::Service<hyper::Request<hyper::body::Incoming>> for RpcClient {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
        let handler = self.clone();
        let future = async move {
            match (req.method(), req.uri().path()) {
                (&hyper::Method::OPTIONS, _) => Ok(RpcResponse::ok(u64::MAX).into()),
                (_, "/") => match req.collect().await {
                    Ok(body) => {
                        let whole_body = body.aggregate();
                        match serde_json::from_reader(whole_body.reader()) {
                            Ok(req) => Ok(handler.handle(req).await.into()),

                            Err(e) => Ok(RpcResponse::error(u64::MAX, &e.to_string()).into()),
                        }
                    }
                    Err(e) => {
                        tracing::error!("error reading request body: {:?}", e);
                        Ok(RpcResponse::error(u64::MAX, &e.to_string()).into())
                    }
                },
                _ => Ok(RpcResponse::error(u64::MAX, "not found").into()),
            }
        };

        Box::pin(future)
    }
}
