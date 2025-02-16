use solana_sdk::pubkey::Pubkey;

pub fn pubkey_serialize<S>(pubkey: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&pubkey.to_string())
}
