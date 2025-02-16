
use std::future::Future;
use std::pin::Pin;

use http_body_util::{BodyExt, Full};
use hyper::body::{Buf, Bytes};
use hyper::Response;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
// HttpSubscriber is a simple HTTP server that listens for POST requests and forwards the body of the request to a tokio mpsc channel.
// This allows for easy channel-based communication between different machines.
// This is useful for testing and debugging.
// This is not production-ready stuff, TCP calls are kinda slow.
pub struct HttpClient<T> {
}


impl<T> HttpClient<T>
where
    T: Message,
{
    pub fn start(port: u16) -> tokio::sync::mpsc::Receiver<T> {
        let (tx, rx) = tokio::sync::mpsc::channel(1000);
        let client = Self { tx };
        tokio::spawn(async move {
            if let Err(e) = client.run(port).await {
                tracing::error!("error running server: {:?}", e);
            }
        });
        rx
    }

    async fn run(self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
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
}

impl<T> hyper::service::Service<hyper::Request<hyper::body::Incoming>> for HttpClient<T>
where
    T: Message + for<'de> Deserialize<'de>,
{
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
        let handler = self.clone();
        let future = async move {
            match (req.method(), req.uri().path()) {
                (&hyper::Method::OPTIONS, _) => Ok(RpcResponse::Ok(()).into()),
                (_, "/") => match req.collect().await {
                    Ok(body) => {
                        let whole_body = body.aggregate();
                        match serde_json::from_reader(whole_body.reader()) {
                            Ok(msg) => {
                                handler.tx.send(msg).await.expect("error sending message");
                                Ok(RpcResponse::Ok(()).into())
                            }
                            Err(e) => {
                                tracing::error!("error deserializing request body: {:?}", e);
                                Ok(RpcResponse::Error(e.to_string()).into())
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("error reading request body: {:?}", e);
                        Ok((RpcResponse::Error(e.to_string())).into())
                    }
                },
                _ => Ok(RpcResponse::Error("not found".to_string()).into()),
            }
        };

        Box::pin(future)
    }
}

#[cfg(test)]
mod tests {
    use crate::http_sender::HttpSender;

    use super::*;

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_sender_receiver_connection() {
        let host = "localhost";
        let port = 1339;
        let addr = format!("http://{}:{}", host, port);

        let sender = HttpSender::<String>::start(&addr);
        let mut receiver = HttpClient::<String>::start(port);

        let msg = "hello world".to_string();

        sender
            .send(msg.clone())
            .await
            .expect("error sending message");
        let received = receiver.recv().await.expect("error receiving message");
        assert_eq!(msg, received);
        tracing::debug!("received: {:?}", received);
    }
}

