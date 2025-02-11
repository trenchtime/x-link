use crate::keygen::Keygen;

#[derive(clap::Parser)]
pub struct Args {
    #[clap(short, long)]
    handle: String,
    #[clap(long)]
    secret_file: String,
    #[clap(long)]
    passphrase: String,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let keygen = Keygen::load(&self.secret_file, &self.passphrase)?;
        let key = keygen.generate_key(&self.handle)?;
        tracing::info!(?key, "Key Generated");
        Ok(())
    }
}
