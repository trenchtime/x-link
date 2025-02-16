use serde::Deserializer;
use solana_sdk::pubkey::Pubkey;

/// Zero-copy deserialization of a base58-encoded 32-byte pubkey.
pub fn pubkey_deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    struct PubkeyVisitor;

    impl serde::de::Visitor<'_> for PubkeyVisitor {
        type Value = Pubkey;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a base58-encoded 32-byte Solana hash")
        }

        // This is called when Serde sees a JSON string and wants us to handle it
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            // Allocate a single [u8; 32] buffer for decoded bytes
            let mut buf = [0u8; 32];

            // Decode directly into the fixed buffer. No extra Vec is created.
            let decoded_len = bs58::decode(v).onto(&mut buf[..]).map_err(E::custom)?;

            if decoded_len != 32 {
                return Err(E::custom(format!("expected 32 bytes, got {}", decoded_len)));
            }

            Ok(Pubkey::from(buf))
        }
    }

    deserializer.deserialize_str(PubkeyVisitor)
}

