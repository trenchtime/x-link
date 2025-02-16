use serde::{ser::SerializeStruct as _, Serialize, Serializer};
use solana_sdk::{signature::Keypair, signer::Signer as _};

pub struct Account {
    pub twitter_id: String,
    pub wallet: Keypair,
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Account", 2)?;
        state.serialize_field("twitter_id", &self.twitter_id)?;
        state.serialize_field("wallet", &self.wallet.pubkey().to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_serialize_account() {
        let account = Account {
            twitter_id: "123456".to_string(),
            wallet: Keypair::new(),
        };

        let expected = json!({
            "twitter_id": "123456",
            "wallet": account.wallet.pubkey().to_string(),
        });

        assert_eq!(serde_json::to_value(&account).unwrap(), expected);
    }
}
