//! Integration tests for riglr-asterdex crate

use core::result::Result;
use mockito::{Matcher, Server};
use riglr_asterdex::client::{AccountBalance, AccountInfo, Order, OrderResponse, Position};
use riglr_asterdex::{Client, Error};
use riglr_config::Config;
use riglr_core::{
    provider::ApplicationContext,
    signer::{
        error::Error as SignerError, Chain, EvmClient, EvmSigner, SignerBase, SolanaSigner,
        UnifiedSigner,
    },
    SignerContext,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;

// Test EVM signer implementation
#[derive(Debug)]
struct TestEvmSigner {
    test_address: String,
    test_chain_id: u64,
}

impl TestEvmSigner {
    fn new() -> Self {
        Self {
            test_address: "0x123_4567890123_4567890123_4567890123_4567890".to_string(),
            test_chain_id: 1,
        }
    }
}

impl SignerBase for TestEvmSigner {
    fn user_id(&self) -> String {
        "test_user".to_string()
    }

    fn supported_chains(&self) -> &[Chain] {
        &[Chain::Evm]
    }
}

#[async_trait::async_trait]
impl EvmSigner for TestEvmSigner {
    fn address(&self) -> String {
        self.test_address.clone()
    }

    fn chain_id(&self) -> u64 {
        self.test_chain_id
    }

    fn client(&self) -> &dyn EvmClient {
        unimplemented!("Not implemented for tests")
    }

    async fn sign_and_send_transaction(
        &self,
        _tx: serde_json::Value,
    ) -> Result<String, Box<dyn SignerError>> {
        Ok("0xtesthash".to_string())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Box<dyn SignerError>> {
        // Return a deterministic mock signature for testing
        Ok("abcd123_4567890abcdef123_4567890abcdef123_4567890abcdef123_4567890abcdef123_4567890abcdef123_4567890abcdef123_4567890abcdef123_4567890".to_string())
    }
}

// Test unified signer wrapper
#[derive(Debug)]
struct TestUnifiedSigner {
    evm: TestEvmSigner,
}

impl TestUnifiedSigner {
    fn new() -> Self {
        Self {
            evm: TestEvmSigner::new(),
        }
    }
}

impl SignerBase for TestUnifiedSigner {
    fn user_id(&self) -> String {
        "test_user".to_string()
    }

    fn supported_chains(&self) -> &[Chain] {
        &[Chain::Evm]
    }
}

impl UnifiedSigner for TestUnifiedSigner {
    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        Some(&self.evm)
    }

    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        None
    }
}

// Helper to create test application context
#[allow(dead_code)]
fn create_test_context() -> ApplicationContext {
    let config = Config::from_env();
    ApplicationContext::from_config(&config)
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_place_order_success() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("POST", "/fapi/v3/order")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .with_status(200)
        .with_body(
            json!({
                "orderId": 123_456,
                "symbol": "BTCUSDT",
                "status": "NEW",
                "clientOrderId": "test123",
                "price": "50000.00",
                "avgPrice": "0.00",
                "origQty": "0.001",
                "executedQty": "0",
                "cumulativeQuoteQty": "0",
                "timeInForce": "GTC",
                "type": "LIMIT",
                "side": "BUY",
                "stopPrice": null,
                "origType": "LIMIT",
                "activatePrice": null,
                "priceRate": null,
                "updateTime": 1_234_567_890_000_i64,
                "workingType": "CONTRACT_PRICE"
            })
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        // Create client with mock server URL
        let client = Client::with_base_url(signer, mock_url).unwrap();

        // We need to test through the actual tool functions
        // However, the tools use SignerContext::current() internally
        // So we're already in a signer context here

        // For now, let's test the client directly
        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("side".to_string(), "BUY".to_string());
        params.insert("quantity".to_string(), "0.001".to_string());
        params.insert("type".to_string(), "LIMIT".to_string());
        params.insert("price".to_string(), "50000.00".to_string());
        params.insert("positionSide".to_string(), "BOTH".to_string());
        params.insert("timeInForce".to_string(), "GTC".to_string());

        let response = client.post("/fapi/v3/order", params).await.unwrap();
        let order: OrderResponse = response.json().await.unwrap();

        assert_eq!(order.order_id, 123_456);
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.status, "NEW");

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_cancel_order_success() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("DELETE", "/fapi/v3/order")
        .with_status(200)
        .with_body(
            json!({
                "orderId": 123_456,
                "symbol": "BTCUSDT",
                "status": "CANCELED",
                "clientOrderId": "test123",
                "price": "50000.00",
                "avgPrice": "0.00",
                "origQty": "0.001",
                "executedQty": "0",
                "cumulativeQuoteQty": "0",
                "timeInForce": "GTC",
                "type": "LIMIT",
                "side": "BUY",
                "positionSide": "BOTH",
                "stopPrice": null,
                "workingType": "CONTRACT_PRICE",
                "origType": "LIMIT",
                "activatePrice": null,
                "priceRate": null,
                "updateTime": 1_234_567_890_000_i64,
                "reduceOnly": false,
                "closePosition": false
            })
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("orderId".to_string(), "123_456".to_string());

        let response = client.delete("/fapi/v3/order", params).await.unwrap();
        let order: Order = response.json().await.unwrap();

        assert_eq!(order.order_id, 123_456);
        assert_eq!(order.status, "CANCELED");

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_get_positions() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("GET", "/fapi/v3/positionRisk")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(
            json!([
                {
                    "symbol": "BTCUSDT",
                    "positionAmt": "0.001",
                    "entryPrice": "50000.00",
                    "markPrice": "51000.00",
                    "unRealizedProfit": "1.00",
                    "liquidationPrice": "45000.00",
                    "leverage": "10",
                    "maxNotionalValue": "100000",
                    "marginType": "cross",
                    "isolatedMargin": "0",
                    "isAutoAddMargin": "false",
                    "positionSide": "BOTH",
                    "notional": "51.00",
                    "isolatedWallet": "0",
                    "updateTime": 1_234_567_890_000_i64
                }
            ])
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let response = client
            .get("/fapi/v3/positionRisk", BTreeMap::new())
            .await
            .unwrap();
        let positions: Vec<Position> = response.json().await.unwrap();

        assert_eq!(positions.len(), 1);
        assert_eq!(positions.first().unwrap().symbol, "BTCUSDT");
        assert_eq!(positions.first().unwrap().position_amt, "0.001");

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_get_account_info() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("GET", "/fapi/v3/account")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(
            json!({
                "feeTier": 0,
                "canTrade": true,
                "canDeposit": true,
                "canWithdraw": true,
                "updateTime": 1_234_567_890_000_i64,
                "totalInitialMargin": "0.00",
                "totalMaintMargin": "0.00",
                "totalWalletBalance": "1000.00",
                "totalUnrealizedProfit": "0.00",
                "totalMarginBalance": "1000.00",
                "totalPositionInitialMargin": "0.00",
                "totalOpenOrderInitialMargin": "0.00",
                "totalCrossWalletBalance": "1000.00",
                "totalCrossUnPnl": "0.00",
                "availableBalance": "1000.00",
                "maxWithdrawAmount": "1000.00",
                "positions": [],
                "assets": [{
                    "asset": "USDT",
                    "walletBalance": "1000.00",
                    "unrealizedProfit": "0.00",
                    "marginBalance": "1000.00",
                    "maintMargin": "0.00",
                    "initialMargin": "0.00",
                    "positionInitialMargin": "0.00",
                    "openOrderInitialMargin": "0.00",
                    "crossWalletBalance": "1000.00",
                    "crossUnPnl": "0.00",
                    "availableBalance": "1000.00",
                    "maxWithdrawAmount": "1000.00",
                    "marginAvailable": true,
                    "updateTime": 1_234_567_890_000_i64
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let response = client
            .get("/fapi/v3/account", BTreeMap::new())
            .await
            .unwrap();
        let account: AccountInfo = response.json().await.unwrap();

        assert!(account.can_trade);
        assert_eq!(account.total_wallet_balance, "1000.00");
        assert_eq!(account.assets.len(), 1);

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_get_balance() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("GET", "/fapi/v3/balance")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(
            json!([
                {
                    "accountAlias": "main",
                    "asset": "USDT",
                    "balance": "1000.00",
                    "crossWalletBalance": "1000.00",
                    "crossUnPnl": "0.00",
                    "availableBalance": "1000.00",
                    "maxWithdrawAmount": "1000.00",
                    "marginAvailable": true,
                    "updateTime": 1_234_567_890_000_i64
                }
            ])
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let response = client
            .get("/fapi/v3/balance", BTreeMap::new())
            .await
            .unwrap();
        let balances: Vec<AccountBalance> = response.json().await.unwrap();

        assert_eq!(balances.len(), 1);
        assert_eq!(balances.first().unwrap().asset, "USDT");
        assert_eq!(balances.first().unwrap().balance, "1000.00");

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_set_leverage() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("POST", "/fapi/v1/leverage")
        .with_status(200)
        .with_body(
            json!({
                "symbol": "BTCUSDT",
                "leverage": 10,
                "maxNotionalValue": "1000000"
            })
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("leverage".to_string(), "10".to_string());

        let response = client.post("/fapi/v1/leverage", params).await.unwrap();
        let result: serde_json::Value = response.json().await.unwrap();

        assert_eq!(result.get("leverage").unwrap(), &10);
        assert_eq!(result.get("symbol").unwrap(), "BTCUSDT");

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used, clippy::panic)]
async fn test_error_handling_rate_limit() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("GET", "/fapi/v3/account")
        .match_query(Matcher::Any)
        .with_status(429)
        .with_body("Rate limit exceeded")
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let result = client.get("/fapi/v3/account", BTreeMap::new()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::RateLimit(msg) => assert!(msg.contains("Rate limit")),
            _ => panic!("Expected RateLimit error, got: {err:?}"),
        }

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used, clippy::panic)]
async fn test_error_handling_auth_error() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("POST", "/fapi/v3/order")
        .with_status(401)
        .with_body("Unauthorized")
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let result = client.post("/fapi/v3/order", BTreeMap::new()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::AuthError(msg) => assert!(msg.contains("Unauthorized")),
            _ => panic!("Expected AuthError, got: {err:?}"),
        }

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used, clippy::panic)]
async fn test_error_handling_invalid_input() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("POST", "/fapi/v3/order")
        .with_status(400)
        .with_body(
            json!({
                "code": -1102,
                "msg": "Mandatory parameter 'symbol' was not sent"
            })
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let result = client.post("/fapi/v3/order", BTreeMap::new()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::InvalidInput(msg) => assert!(msg.contains("Mandatory parameter")),
            _ => panic!("Expected InvalidInput error, got: {err:?}"),
        }

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used, clippy::panic)]
async fn test_error_handling_insufficient_balance() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("POST", "/fapi/v3/order")
        .with_status(400)
        .with_body(
            json!({
                "code": -2010,
                "msg": "Insufficient balance"
            })
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let result = client.post("/fapi/v3/order", BTreeMap::new()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::InsufficientBalance(msg) => assert!(msg.contains("Insufficient balance")),
            _ => panic!("Expected InsufficientBalance error, got: {err:?}"),
        }

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used, clippy::panic)]
async fn test_error_handling_server_error() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("GET", "/fapi/v3/account")
        .match_query(Matcher::Any)
        .with_status(500)
        .with_body("Internal server error")
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let result = client.get("/fapi/v3/account", BTreeMap::new()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::ApiError(msg) => assert!(msg.contains("Server error")),
            _ => panic!("Expected ApiError for server error, got: {err:?}"),
        }

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_get_open_orders() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("GET", "/fapi/v3/openOrders")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(
            json!([
                {
                    "orderId": 123_456,
                    "symbol": "BTCUSDT",
                    "status": "NEW",
                    "clientOrderId": "test123",
                    "price": "50000.00",
                    "avgPrice": "0.00",
                    "origQty": "0.001",
                    "executedQty": "0",
                    "cumulativeQuoteQty": "0",
                    "timeInForce": "GTC",
                    "type": "LIMIT",
                    "side": "BUY",
                    "positionSide": "BOTH",
                    "stopPrice": null,
                    "workingType": "CONTRACT_PRICE",
                    "origType": "LIMIT",
                    "activatePrice": null,
                    "priceRate": null,
                    "updateTime": 1_234_567_890_000_i64,
                    "reduceOnly": false,
                    "closePosition": false
                }
            ])
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let response = client
            .get("/fapi/v3/openOrders", BTreeMap::new())
            .await
            .unwrap();
        let orders: Vec<Order> = response.json().await.unwrap();

        assert_eq!(orders.len(), 1);
        assert_eq!(orders.first().unwrap().order_id, 123_456);
        assert_eq!(orders.first().unwrap().status, "NEW");

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_get_specific_order() {
    let mut server = Server::new_async().await;
    let mock_url = server.url();

    let _m = server
        .mock("GET", "/fapi/v3/order")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(
            json!({
                "orderId": 123_456,
                "symbol": "BTCUSDT",
                "status": "FILLED",
                "clientOrderId": "test123",
                "price": "50000.00",
                "avgPrice": "50000.00",
                "origQty": "0.001",
                "executedQty": "0.001",
                "cumulativeQuoteQty": "50.00",
                "timeInForce": "GTC",
                "type": "LIMIT",
                "side": "BUY",
                "positionSide": "BOTH",
                "stopPrice": null,
                "workingType": "CONTRACT_PRICE",
                "origType": "LIMIT",
                "activatePrice": null,
                "priceRate": null,
                "updateTime": 1_234_567_890_000_i64,
                "reduceOnly": false,
                "closePosition": false
            })
            .to_string(),
        )
        .create_async()
        .await;
    drop(server);

    let signer = Arc::new(TestUnifiedSigner::new());

    SignerContext::with_signer(signer.clone(), async move {
        let client = Client::with_base_url(signer, mock_url).unwrap();

        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("orderId".to_string(), "123_456".to_string());

        let response = client.get("/fapi/v3/order", params).await.unwrap();
        let order: Order = response.json().await.unwrap();

        assert_eq!(order.order_id, 123_456);
        assert_eq!(order.status, "FILLED");
        assert_eq!(order.executed_qty, "0.001");

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .unwrap();
}
