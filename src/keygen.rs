use sha2::Digest;
use solana_sdk::signer::{keypair::Keypair, SeedDerivable as _, Signer as _};

pub struct Keygen([u8; 64]);

impl std::ops::Deref for Keygen {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u8; 64]> for Keygen {
    fn from(value: [u8; 64]) -> Self {
        Self(value)
    }
}

impl Keygen {
    pub fn load(secret_file: &str, passphrase: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let secret = std::fs::read_to_string(secret_file)?.trim().to_string();
        let seed = bip39::Mnemonic::parse(secret)?.to_seed(passphrase);
        Ok(Self::from(seed))
    }

    pub fn hash(handle: &str) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(handle.as_bytes());
        hasher.finalize().into()
    }

    pub fn seed_for_handle(
        secret: [u8; 64],
        handle: &str,
    ) -> Result<[u8; 96], Box<dyn std::error::Error>> {
        let mut seed = [0u8; 96];
        seed[..64].copy_from_slice(&secret);
        seed[64..96].copy_from_slice(&Self::hash(handle));
        Ok(seed)
    }

    pub fn generate_key_inner(
        secret: [u8; 64],
        handle: &str,
    ) -> Result<Keypair, Box<dyn std::error::Error>> {
        Keypair::from_seed(Self::seed_for_handle(secret, handle)?.as_slice())
    }

    pub fn generate_key(&self, handle: &str) -> Result<XKey, Box<dyn std::error::Error>> {
        Self::generate_key_inner(**self, handle).map(|key| XKey::new(handle, key))
    }
}

pub struct XKey {
    pub handle: String,
    pub keypair: Keypair,
}

impl std::fmt::Debug for XKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XKey")
            .field("handle", &self.handle)
            .field("key", &self.keypair.pubkey())
            .finish()
    }
}

impl XKey {
    pub fn new(handle: &str, keypair: Keypair) -> XKey {
        XKey {
            handle: handle.to_string(),
            keypair,
        }
    }
}

impl std::ops::Deref for XKey {
    type Target = Keypair;

    fn deref(&self) -> &Self::Target {
        &self.keypair
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use super::*;

    #[test]
    fn test_x_key() {
        const HANDLE: &str = "@burckmeister";
        const SECRET: &[u8; 64] =
            b"What the fuck did you just fucking say about me you little bitch";
        const EXPECTED: Pubkey =
            Pubkey::from_str_const("2gs6bd3SieSs5KaE92c1d8RcnDk38AR2AMgJPvJCazTP");

        let keygen = Keygen::from(*SECRET);
        let keypair = keygen.generate_key(HANDLE).expect("Error generating key");
        assert_eq!(keypair.pubkey(), EXPECTED);
    }
}
