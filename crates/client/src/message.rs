use crate::deserialize::pubkey_deserialize;
use crate::serialize::pubkey_serialize;
use http_body_util::Full;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use x_link_solana::QuoteResponse;
use x_link_types::account::Account;

#[derive(Serialize, Debug)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(flatten)]
    pub params: RpcParams,
}

impl<'de> Deserialize<'de> for RpcRequest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        #[derive(Deserialize, Debug)]
        struct RawRequest {
            jsonrpc: String,
            id: u64,
            method: String,
            params: serde_json::Value,
        }

        let raw = RawRequest::deserialize(deserializer)?;

        let params = match raw.method.as_str() {
            "buy" => serde_json::from_value(raw.params)
                .map(RpcParams::Buy)
                .map_err(|e| D::Error::custom(format!("invalid buy params: {}", e)))?,
            "sell" => serde_json::from_value(raw.params)
                .map(RpcParams::Sell)
                .map_err(|e| D::Error::custom(format!("invalid sell params: {}", e)))?,
            "create" => serde_json::from_value(raw.params)
                .map(RpcParams::Create)
                .map_err(|e| D::Error::custom(format!("invalid create params: {}", e)))?,
            "getAccount" => serde_json::from_value(raw.params)
                .map(RpcParams::GetAccount)
                .map_err(|e| D::Error::custom(format!("invalid getAccount params: {}", e)))?,
            _ => return Err(D::Error::custom(format!("invalid method: {}", raw.method))),
        };

        Ok(RpcRequest {
            jsonrpc: raw.jsonrpc,
            id: raw.id,
            method: raw.method,
            params,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<RpcResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl From<Box<dyn std::error::Error>> for RpcResponse {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Self::error(u64::MAX, &e.to_string())
    }
}

impl RpcResponse {
    pub fn error(id: u64, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(RpcError::Error(message.to_string())),
        }
    }

    pub fn ok(id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(RpcResult::Ok),
            error: None,
        }
    }

    pub fn with_account(mut self, account: Account) -> Self {
        self.result = Some(RpcResult::Account(account));
        self
    }

    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.result = Some(RpcResult::Signature(signature));
        self
    }

    pub fn with_quote(mut self, quote: QuoteResponse) -> Self {
        self.result = Some(RpcResult::Quote(quote));
        self
    }
}

#[derive(Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum RpcResult {
    #[serde(rename = "ok")]
    Ok,
    Account(Account),
    Signature(Signature),
    Quote(QuoteResponse),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum RpcError {
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "camelCase")]
pub enum RpcParams {
    Buy(BuyParams),
    Sell(SellParams),
    Create(CreateParams),
    GetAccount(GetAccountParams),
    Quote(QuoteParams),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountParams {
    pub twitter_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuyParams {
    pub twitter_id: u64,
    #[serde(deserialize_with = "pubkey_deserialize")]
    #[serde(serialize_with = "pubkey_serialize")]
    pub token_id: Pubkey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SellParams {
    pub twitter_id: u64,
    #[serde(deserialize_with = "pubkey_deserialize")]
    #[serde(serialize_with = "pubkey_serialize")]
    pub token_id: Pubkey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteParams {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateParams {
    pub twitter_id: u64,
    pub amount: u64,
    pub token: TokenParams,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenParams {
    pub name: String,
    pub ticker: String,
    pub uri: String,
    pub description: String,
}

// TODO: Expand this with more expressive errors
impl From<RpcResponse> for hyper::Response<Full<Bytes>> {
    fn from(res: RpcResponse) -> Self {
        let mut response = hyper::Response::new(Full::from(
            serde_json::to_vec(&res).expect("error serializing response"),
        ));
        *response.status_mut() = match (res.result, res.error) {
            (Some(RpcResult::Ok), None) => hyper::StatusCode::OK,
            (_, Some(RpcError::Error(_))) => hyper::StatusCode::BAD_REQUEST,
            _ => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        };
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_buy_request() {
        let token_id = Pubkey::new_unique();
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "buy",
            "params": {
                "twitterId": "123456",
                "tokenId": token_id.to_string(),
                "amount": 100
            }
        });

        let request: RpcRequest = serde_json::from_value(request_json.clone()).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, 1);
        assert_eq!(request.method, "buy");

        match request.params {
            RpcParams::Buy(ref params) => {
                assert_eq!(params.twitter_id, 123456);
                assert_eq!(params.token_id, token_id);
                assert_eq!(params.amount, 100);
            }
            _ => panic!("Expected Buy params"),
        }

        // Test serialization
        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(serialized, request_json);
    }

    #[test]
    fn test_sell_request() {
        let token_id = Pubkey::new_unique();
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "sell",
            "params": {
                "twitterId": "789012",
                "tokenId": token_id.to_string(),
                "amount": 50
            }
        });

        let request: RpcRequest = serde_json::from_value(request_json.clone()).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, 2);
        assert_eq!(request.method, "sell");

        match request.params {
            RpcParams::Sell(ref params) => {
                assert_eq!(params.twitter_id, 789012);
                assert_eq!(params.token_id, token_id);
                assert_eq!(params.amount, 50);
            }
            _ => panic!("Expected Sell params"),
        }

        // Test serialization
        let serialized = serde_json::to_value(request).unwrap();
        assert_eq!(serialized, request_json);
    }

    #[test]
    fn test_create_request() {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "create",
            "params": {
                "twitterId": "345678",
                "amount": 1000,
                "token": {
                    "name": "Test Token",
                    "ticker": "TEST",
                    "uri": "https://example.com/token",
                    "description": "Test token description"
                }
            }
        });

        let request: RpcRequest = serde_json::from_value(request_json.clone()).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, 3);
        assert_eq!(request.method, "create");

        match request.params {
            RpcParams::Create(ref params) => {
                assert_eq!(params.twitter_id, 345678);
                assert_eq!(params.amount, 1000);
                assert_eq!(params.token.name, "Test Token");
                assert_eq!(params.token.ticker, "TEST");
                assert_eq!(params.token.uri, "https://example.com/token");
                assert_eq!(params.token.description, "Test token description");
            }
            _ => panic!("Expected Create params"),
        }

        // Test serialization
        let serialized = serde_json::to_value(request).unwrap();
        assert_eq!(serialized, request_json);
    }

    #[test]
    fn test_response_ok() {
        let expected_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "ok"
        });

        let response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(RpcResult::Ok),
            error: None,
        };

        let serialized = serde_json::to_value(&response).unwrap();
        assert_eq!(serialized, expected_json);
    }

    #[test]
    fn test_response_error() {
        let expected_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": "Operation failed"
        });

        let response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: None,
            error: Some(RpcError::Error("Operation failed".to_string())),
        };

        let serialized = serde_json::to_value(&response).unwrap();
        assert_eq!(serialized, expected_json);
    }
}
