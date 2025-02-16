use crate::deserialize::pubkey_deserialize;
use crate::serialize::pubkey_serialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use x_link_types::account::Account;

#[derive(Serialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(flatten)]
    pub params: Params,
}

impl<'de> Deserialize<'de> for RpcRequest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        #[derive(Deserialize)]
        struct RawRequest {
            jsonrpc: String,
            id: u64,
            method: String,
            params: serde_json::Value,
        }

        let raw = RawRequest::deserialize(deserializer)?;

        let params = match raw.method.as_str() {
            "buy" => serde_json::from_value(raw.params)
                .map(Params::Buy)
                .map_err(|e| D::Error::custom(format!("invalid buy params: {}", e)))?,
            "sell" => serde_json::from_value(raw.params)
                .map(Params::Sell)
                .map_err(|e| D::Error::custom(format!("invalid sell params: {}", e)))?,
            "create" => serde_json::from_value(raw.params)
                .map(Params::Create)
                .map_err(|e| D::Error::custom(format!("invalid create params: {}", e)))?,
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
pub struct Response {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<RpcResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum RpcResult {
    Account(Account),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum RpcError {
    Error(String),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "camelCase")]
pub enum Params {
    Buy(BuyParams),
    Sell(SellParams),
    Create(CreateParams),
    GetAccount(GetAccountParams),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountParams {
    pub twitter_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyParams {
    pub twitter_id: String,
    #[serde(deserialize_with = "pubkey_deserialize")]
    #[serde(serialize_with = "pubkey_serialize")]
    pub token_id: Pubkey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellParams {
    pub twitter_id: String,
    #[serde(deserialize_with = "pubkey_deserialize")]
    #[serde(serialize_with = "pubkey_serialize")]
    pub token_id: Pubkey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateParams {
    pub twitter_id: String,
    pub amount: u64,
    pub token: TokenParams,
}

#[derive(Serialize, Deserialize)]
pub struct TokenParams {
    pub name: String,
    pub ticker: String,
    pub uri: String,
    pub description: String,
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
            Params::Buy(ref params) => {
                assert_eq!(params.twitter_id, "123456");
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
            Params::Sell(ref params) => {
                assert_eq!(params.twitter_id, "789012");
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
            Params::Create(ref params) => {
                assert_eq!(params.twitter_id, "345678");
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
        let response_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "status": "ok"
            }
        });

        let response: Response = serde_json::from_value(response_json.clone()).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);

        match response.result {
            Some(RpcResult::Account(ref account)) => {
                assert_eq!(account.status, "ok");
            }
            _ => panic!("Expected Account result"),
        }

        // Test serialization
        let serialized = serde_json::to_value(&response).unwrap();
        assert_eq!(serialized, response_json);
    }

    #[test]
    fn test_response_error() {
        let response_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "status": "error",
                "Error": "Operation failed"
            }
        });

        let response: Response = serde_json::from_value(response_json.clone()).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);

        match response.error {
            Some(RpcError::Error(ref err)) => {
                assert_eq!(err, "Operation failed");
            }
            _ => panic!("Expected Error result"),
        }

        // Test serialization
        let serialized = serde_json::to_value(response).unwrap();
        assert_eq!(serialized, response_json);
    }
}

//impl From<RpcResponse> for hyper::Response<Full<Bytes>> {
//    fn from(res: RpcResponse) -> Self {
//        let mut response = hyper::Response::new(Full::from(
//            serde_json::to_vec(&res).expect("error serializing response"),
//        ));
//        *response.status_mut() = match res {
//            RpcResponse::Ok(_) => hyper::StatusCode::OK,
//            RpcResponse::Error(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
//        };
//        response
//    }
//}
