//! Integration tests for BinanceStream
//! Tests the actual Binance WebSocket stream connection

use riglr_streams::core::{operators::ComposableStream, Stream};
use riglr_streams::external::binance::{BinanceConfig, BinanceEventData, BinanceStream};
use std::time::Duration;
use tokio::time::timeout;

/// Test configuration for Binance testnet
fn get_test_config() -> BinanceConfig {
    BinanceConfig {
        streams: vec![
            "btcusdt@ticker".to_string(), // BTC/USDT ticker
            "ethusdt@ticker".to_string(), // ETH/USDT ticker
            "btcusdt@depth5".to_string(), // Order book depth
        ],
        testnet: true, // Use testnet for testing
        buffer_size: 10000,
    }
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_stream_connection() {
    let mut stream = BinanceStream::new("test-binance");
    let config = get_test_config();

    // Start the stream
    match stream.start(config).await {
        Ok(_) => println!("Successfully connected to Binance stream"),
        Err(e) => {
            println!("Failed to connect to Binance stream: {:?}", e);
            // This might happen if Binance is down or rate limiting
            return;
        }
    }

    // Check if stream is running
    assert!(stream.is_running(), "Stream should be running after start");

    // Check health
    let health = stream.health().await;
    assert!(health.is_connected, "Stream should report as connected");

    // Subscribe and wait for at least one event
    let mut rx = stream.subscribe();

    match timeout(Duration::from_secs(5), rx.recv()).await {
        Ok(Ok(event)) => {
            println!("Received Binance event");
            assert!(event.stream_meta.stream_source.contains("binance"));

            // Check event data type
            match &event.data {
                BinanceEventData::Ticker(ticker) => {
                    println!("Ticker: {} @ {}", ticker.symbol, ticker.close_price);
                }
                BinanceEventData::OrderBook(book) => {
                    println!(
                        "Order book: {} with {} bids, {} asks",
                        book.symbol,
                        book.bids.len(),
                        book.asks.len()
                    );
                }
                _ => {}
            }
        }
        Ok(Err(e)) => println!("Error receiving event: {:?}", e),
        Err(_) => panic!("Timeout waiting for events"),
    }

    // Stop the stream
    stream.stop().await.unwrap();
    assert!(
        !stream.is_running(),
        "Stream should not be running after stop"
    );
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_ticker_stream() {
    let mut stream = BinanceStream::new("test-binance-ticker");
    let config = BinanceConfig {
        streams: vec!["btcusdt@ticker".to_string(), "ethusdt@ticker".to_string()],
        testnet: true,
        buffer_size: 1000,
    };

    // Start the stream
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Binance");
        return;
    }

    let mut rx = stream.subscribe();
    let mut btc_tickers = 0;
    let mut eth_tickers = 0;

    // Collect ticker events for 5 seconds
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        match timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(event)) => {
                if let BinanceEventData::Ticker(ticker) = &event.data {
                    if ticker.symbol == "BTCUSDT" {
                        btc_tickers += 1;
                        println!("BTC price: {}", ticker.close_price);
                    } else if ticker.symbol == "ETHUSDT" {
                        eth_tickers += 1;
                        println!("ETH price: {}", ticker.close_price);
                    }
                }

                if btc_tickers >= 2 && eth_tickers >= 2 {
                    break;
                }
            }
            _ => continue,
        }
    }

    assert!(btc_tickers > 0, "Should receive BTC ticker updates");
    assert!(eth_tickers > 0, "Should receive ETH ticker updates");

    stream.stop().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_order_book_stream() {
    let mut stream = BinanceStream::new("test-binance-orderbook");
    let config = BinanceConfig {
        streams: vec![
            "btcusdt@depth5".to_string(), // Top 5 levels
        ],
        testnet: true,
        buffer_size: 1000,
    };

    // Start the stream
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Binance");
        return;
    }

    let mut rx = stream.subscribe();
    let mut books_received = 0;

    // Collect order book updates
    while books_received < 3 {
        match timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Ok(event)) => {
                if let BinanceEventData::OrderBook(book) = &event.data {
                    books_received += 1;

                    assert_eq!(book.symbol, "BTCUSDT");
                    assert!(book.bids.len() <= 5, "Should have at most 5 bid levels");
                    assert!(book.asks.len() <= 5, "Should have at most 5 ask levels");

                    if !book.bids.is_empty() && !book.asks.is_empty() {
                        let best_bid = &book.bids[0];
                        let best_ask = &book.asks[0];
                        println!(
                            "BTC Order Book - Bid: {} @ {}, Ask: {} @ {}",
                            best_bid[1], best_bid[0], best_ask[1], best_ask[0]
                        );
                    }
                }
            }
            Ok(Err(e)) => println!("Error receiving event: {:?}", e),
            Err(_) => {
                println!("Timeout waiting for order book updates");
                break;
            }
        }
    }

    assert!(books_received > 0, "Should receive order book updates");
    stream.stop().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_stream_with_batch() {
    let mut stream = BinanceStream::new("test-binance-batch");
    let config = BinanceConfig {
        streams: vec!["btcusdt@ticker".to_string(), "ethusdt@ticker".to_string()],
        testnet: true,
        buffer_size: 1000,
    };

    // Start the stream first
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Binance");
        return;
    }

    // Batch events in groups of 5 or every 2 seconds
    let batched_stream = stream.batch(5, Duration::from_secs(2));

    let mut rx = batched_stream.subscribe();

    // Wait for at least one batch
    match timeout(Duration::from_secs(10), rx.recv()).await {
        Ok(Ok(batch)) => {
            println!("Received batch with {} Binance events", batch.events.len());
            assert!(!batch.events.is_empty(), "Batch should not be empty");
            assert!(
                batch.events.len() <= 5,
                "Batch should not exceed size limit"
            );
        }
        Ok(Err(e)) => println!("Error receiving batch: {:?}", e),
        Err(_) => println!("Timeout waiting for batch"),
    }

    // Note: stream was moved when creating batched_stream, so we can't stop it here
    // In a real application, you would need to keep a reference to stop the stream
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_stream_reconnection() {
    let mut stream = BinanceStream::new("test-binance-reconnect");
    let config = BinanceConfig {
        streams: vec!["btcusdt@ticker".to_string()],
        testnet: true,
        buffer_size: 1000,
    };

    // Start the stream
    if stream.start(config.clone()).await.is_err() {
        println!("Skipping test - cannot connect to Binance");
        return;
    }

    // Get initial health
    let health1 = stream.health().await;
    assert!(health1.is_connected);

    // Stop the stream
    stream.stop().await.unwrap();

    // Health should show disconnected
    let health2 = stream.health().await;
    assert!(!health2.is_connected);

    // Restart the stream
    match stream.start(config).await {
        Ok(_) => {
            let health3 = stream.health().await;
            assert!(health3.is_connected, "Should reconnect successfully");
        }
        Err(e) => println!("Failed to reconnect: {:?}", e),
    }

    stream.stop().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_stream_metrics() {
    let mut stream = BinanceStream::new("test-binance-metrics");
    let config = BinanceConfig {
        streams: vec!["btcusdt@ticker".to_string(), "ethusdt@ticker".to_string()],
        testnet: true,
        buffer_size: 1000,
    };

    // Start the stream
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Binance");
        return;
    }

    let mut rx = stream.subscribe();
    let mut events_received = 0;

    // Collect some events
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        match timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(_event)) => {
                events_received += 1;
                if events_received >= 10 {
                    break;
                }
            }
            _ => continue,
        }
    }

    // Check health metrics
    let health = stream.health().await;
    assert!(health.is_connected);
    assert!(
        health.events_processed > 0,
        "Should have processed some events"
    );
    assert!(
        health.last_event_time.is_some(),
        "Should have last event time"
    );

    println!(
        "Binance stream metrics - Events processed: {}, Errors: {}",
        health.events_processed, health.error_count
    );

    stream.stop().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_kline_stream() {
    let mut stream = BinanceStream::new("test-binance-kline");
    let config = BinanceConfig {
        streams: vec![
            "btcusdt@kline_1m".to_string(), // 1-minute candles
        ],
        testnet: true,
        buffer_size: 1000,
    };

    // Start the stream
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Binance");
        return;
    }

    let mut rx = stream.subscribe();
    let mut klines_received = 0;

    // Collect kline events
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        match timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Ok(event)) => {
                if let BinanceEventData::Kline(kline) = &event.data {
                    klines_received += 1;
                    println!(
                        "BTC 1m Kline - Open: {}, High: {}, Low: {}, Close: {}",
                        kline.kline.open, kline.kline.high, kline.kline.low, kline.kline.close
                    );

                    if klines_received >= 2 {
                        break;
                    }
                }
            }
            _ => continue,
        }
    }

    assert!(klines_received > 0, "Should receive kline updates");
    stream.stop().await.unwrap();
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_binance_trade_stream() {
    let mut stream = BinanceStream::new("test-binance-trade");
    let config = BinanceConfig {
        streams: vec![
            "btcusdt@trade".to_string(), // Individual trades
        ],
        testnet: true,
        buffer_size: 1000,
    };

    // Start the stream
    if stream.start(config).await.is_err() {
        println!("Skipping test - cannot connect to Binance");
        return;
    }

    let mut rx = stream.subscribe();
    let mut trades_received = 0;

    // Collect trade events
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        match timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(event)) => {
                if let BinanceEventData::Trade(trade) = &event.data {
                    trades_received += 1;
                    println!(
                        "BTC Trade - Price: {}, Quantity: {}, Buyer is maker: {}",
                        trade.price, trade.quantity, trade.is_buyer_maker
                    );

                    if trades_received >= 5 {
                        break;
                    }
                }
            }
            _ => continue,
        }
    }

    assert!(trades_received > 0, "Should receive trade updates");
    stream.stop().await.unwrap();
}
