use crate::{client::RpcClient, error::Error};

#[derive(clap::Parser)]
pub struct Args {
    #[clap(long)]
    secret_file: String,

    #[clap(long, default_value = "1337")]
    port: u16,
}

impl Args {
    pub async fn run(&self) -> Result<(), Error> {
        RpcClient::start(&self.secret_file, self.port).await
    }
}
