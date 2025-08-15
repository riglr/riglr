use std::sync::Arc;
use std::time::Duration;
use riglr_core::transactions::{GenericTransactionProcessor, TransactionProcessor, RetryConfig, TransactionStatus};
use riglr_core::error::ToolError;

#[tokio::test]
async fn generic_processor_retry_and_status() {
    let proc = GenericTransactionProcessor;
    let attempts = Arc::new(std::sync::Mutex::new(0));
    let a2 = attempts.clone();

    // Fails once with retriable, then succeeds
    let out = proc.process_with_retry(
        move || {
            let a2 = a2.clone();
            async move {
                let mut c = a2.lock().unwrap();
                *c += 1;
                if *c < 2 {
                    Err(ToolError::retriable_string("temporary"))
                } else {
                    Ok("ok")
                }
            }
        },
        RetryConfig { max_attempts: 3, initial_delay: Duration::from_millis(1), max_delay: Duration::from_millis(5), backoff_multiplier: 2.0 }
    ).await.unwrap();
    assert_eq!(out, "ok");

    // Non-retriable error returns immediately
    let err = proc.process_with_retry(
        || async { Err(ToolError::permanent_string("bad")) },
        RetryConfig::default(),
    ).await.unwrap_err();
    assert!(!err.is_retriable());

    // get_status empty hash should error
    let st = proc.get_status("").await;
    assert!(st.is_err());
}

#[tokio::test]
async fn wait_for_confirmation_paths() {
    struct TestProc;
    #[async_trait::async_trait]
    impl TransactionProcessor for TestProc {
        async fn process_with_retry<T, F, Fut>(&self, _op: F, _cfg: RetryConfig) -> Result<T, ToolError>
        where T: Send, F: Fn() -> Fut + Send, Fut: std::future::Future<Output = Result<T, ToolError>> + Send {
            unimplemented!()
        }
        async fn get_status(&self, tx_hash: &str) -> Result<TransactionStatus, ToolError> {
            match tx_hash {
                "ok" => Ok(TransactionStatus::Confirmed { hash: tx_hash.into(), block: 1 }),
                "fail" => Ok(TransactionStatus::Failed { reason: "x".into() }),
                _ => Ok(TransactionStatus::Submitted { hash: tx_hash.into() }),
            }
        }
        async fn wait_for_confirmation(&self, tx_hash: &str, required: u64) -> Result<TransactionStatus, ToolError> {
            // Delegate to generic for loop/wait behavior
            GenericTransactionProcessor.wait_for_confirmation(tx_hash, required).await
        }
    }

    let tp = TestProc;
    assert!(matches!(tp.wait_for_confirmation("ok", 1).await.unwrap(), TransactionStatus::Confirmed{..}));
    let err = tp.wait_for_confirmation("fail", 1).await.unwrap_err();
    assert!(!err.is_retriable());
}
