use sha2::Digest;
use solana_sdk::{derivation_path::DerivationPath, signature::Keypair, signer::SeedDerivable as _};

pub struct KeyGen([u8; 64]);

impl std::ops::Deref for KeyGen {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u8; 64]> for KeyGen {
    fn from(value: [u8; 64]) -> Self {
        Self(value)
    }
}

pub trait KeyGenerator<T> {
    fn generate_key(&self, data: T) -> Result<Keypair, Box<dyn std::error::Error>>;
}

impl KeyGenerator<u64> for KeyGen {
    fn generate_key(&self, id: u64) -> Result<Keypair, Box<dyn std::error::Error>> {
        Self::key_from_id_inner(**self, id)
    }
}

impl KeyGenerator<&str> for KeyGen {
    fn generate_key(&self, handle: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
        Self::key_from_handle_inner(**self, handle)
    }
}

impl KeyGen {
    pub fn load(secret_file: &str, passphrase: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let secret = std::fs::read_to_string(secret_file)?.trim().to_string();
        let seed = bip39::Mnemonic::parse(secret)?.to_seed(passphrase);
        Ok(Self::from(seed))
    }

    fn seed_for_handle(secret: [u8; 64], handle: &str) -> [u8; 64] {
        let mut hasher = sha2::Sha512::new();
        hasher.update(secret);
        hasher.update(handle.as_bytes());
        hasher.finalize().into()
    }

    fn key_from_handle_inner(
        secret: [u8; 64],
        handle: &str,
    ) -> Result<Keypair, Box<dyn std::error::Error>> {
        Keypair::from_seed(&Self::seed_for_handle(secret, handle))
    }

    fn key_from_id_inner(secret: [u8; 64], id: u64) -> Result<Keypair, Box<dyn std::error::Error>> {
        let id_bytes = id.to_be_bytes();
        let account = u32::from_be_bytes(id_bytes[0..4].try_into()?);
        let change = u32::from_be_bytes(id_bytes[4..8].try_into()?);
        let derivation_path = DerivationPath::new_bip44(Some(account), Some(change));
        Keypair::from_seed_and_derivation_path(&secret, Some(derivation_path))
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signer::Signer as _;

    use super::*;

    #[test]
    fn test_x_key_from_id() {
        const SECRET: &[u8; 64] =
            b"What the fuck did you just fucking say about me you little bitch";
        const EXPECTED: Pubkey = pubkey!("ENuzcbEgZq9j9BQCSgsFfvnMeX8aY7xGDizGyf29eByN");

        const ID: u64 = 1722992406616756224;
        const OTHER_HANDLE: u64 = ID + 1;

        let keygen = KeyGen::from(*SECRET);

        let keypair = keygen.generate_key(ID).expect("Error generating key");
        assert_eq!(keypair.pubkey(), EXPECTED);

        let other_keypair = keygen
            .generate_key(OTHER_HANDLE)
            .expect("Error generating key");
        assert_ne!(other_keypair.pubkey(), EXPECTED);
    }
}
