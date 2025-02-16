use x_link_wallet::keygen::{KeyGen, KeyGenerator};

#[derive(clap::Parser)]
pub struct Args {
    #[clap(short, long)]
    id: u64,
    #[clap(long)]
    secret_file: String,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let passphrase = rpassword::prompt_password("Enter passphrase: ")?;
        let keygen = KeyGen::load(&self.secret_file, &passphrase)?;
        let key = keygen.generate_key(self.id)?;
        tracing::info!(?key, "Key Generated");
        Ok(())
    }
}
