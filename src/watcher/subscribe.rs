use std::sync::{atomic, Arc};

use crate::subscribe_filter::ShallowFilter;
use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::Transaction,
};
use eyre::Result;
use futures_util::StreamExt;
use tokio::{sync::Mutex, task::JoinHandle};

pub struct ShutdownConfig {
    pub flag: Arc<atomic::AtomicBool>,
    pub result: Arc<atomic::AtomicBool>, // true if stream finished
}

pub async fn subscribe_to_pending<P, T>(
    provider: Arc<P>,
    filter_type: T,
    tx_buffer: Arc<Mutex<Vec<Transaction>>>,
    shutdown_config: ShutdownConfig,
) -> Result<JoinHandle<()>>
where
    P: Provider<PubSubFrontend> + Send + Sync + 'static,
    T: ShallowFilter + Clone + Send + Sync + 'static,
{
    let sub = provider.subscribe_pending_transactions().await.unwrap();

    // Filter stream based on filter type
    let stream = sub.into_stream().filter_map(move |tx_hash| {
        let provider = provider.clone();
        let filter_type = filter_type.clone();
        async move {
            match provider.get_transaction_by_hash(tx_hash).await {
                Ok(tx) => tx.and_then(|tx| {
                    if filter_type.filter(&tx) {
                        Some(tx)
                    } else {
                        None
                    }
                }),
                Err(e) => {
                    println!("Error: {:#?}", e);
                    None
                }
            }
        }
    });
    println!("Awaiting pending transactions...");

    // Pin the stream for use in the async block
    let pinned_stream = Box::pin(stream);

    // Take the stream and print the pending transaction.
    let handle = tokio::spawn(async move {
        let mut stream = pinned_stream; // Store the stream in a pinned variable
        loop {
            tokio::select! {
                tx = stream.next() => {
                    if shutdown_config.flag.load(atomic::Ordering::SeqCst) {
                        break;
                    }
                    if let Some(tx) = tx {
                        // Add transaction to buffer
                        println!("Pending transaction: {:#?}", tx);
                        let mut buffer = tx_buffer.lock().await;
                        buffer.push(tx);
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    if shutdown_config.flag.load(atomic::Ordering::SeqCst) {
                        break;
                    }
                }
            }
        }
        shutdown_config.result.store(true, atomic::Ordering::SeqCst);
    });

    Ok(handle)
}

pub async fn create_ws_provider(rpc_url: &str) -> Result<impl Provider<PubSubFrontend>> {
    let ws = WsConnect::new(rpc_url);
    ProviderBuilder::new()
        .on_ws(ws)
        .await
        .map_or_else(|e| Err(e.into()), Ok)
}

#[cfg(test)]
mod tests {

    use alloy::{network::TransactionBuilder, primitives::U256, rpc::types::TransactionRequest};
    use subscribe_filter::ShallowFilterType;

    use crate::{setup, subscribe_filter, testconfig};

    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_subscribe_to_pending() {
        let config = testconfig::TestConfig::load();

        let ws_provider = create_ws_provider(&config.anvil_ws_endpoint).await.unwrap();

        let (http_provider, http_addr) = setup::test_setup().await;

        let provider = Arc::new(ws_provider);
        let tx_buffer = Arc::new(Mutex::new(Vec::new()));
        let shutdown_flag = Arc::new(atomic::AtomicBool::new(false)); // Added shutdown flag
        let shutdown_result = Arc::new(atomic::AtomicBool::new(false)); // Added result flag

        // Start monitoring pending transactions
        let _handle = subscribe_to_pending(
            provider.clone(),
            ShallowFilterType::Recipient(http_addr),
            tx_buffer.clone(),
            ShutdownConfig {
                flag: shutdown_flag.clone(),
                result: shutdown_result.clone(),
            },
        )
        .await
        .unwrap();

        // Allow subscription to initialize
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Send a transaction to ourselves
        for _ in 0..3 {
            let _ = http_provider
                .send_transaction(
                    TransactionRequest::default()
                        .with_to(http_addr)
                        .with_value(U256::from(100)),
                )
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();
        }

        // Allow some time for processing the transaction
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Verify that a transaction has been received
        {
            let buffer = tx_buffer.lock().await;
            // Assert size of buffer
            assert_eq!(buffer.len(), 3, "Buffer should contain 3 transactions");
        }

        // Verify that the shutdown result has not yet been set
        assert!(
            !shutdown_result.load(atomic::Ordering::SeqCst),
            "Stream should not have finished yet"
        );

        // Signal shutdown
        shutdown_flag.store(true, atomic::Ordering::SeqCst);

        // Wait for the stream to finish
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Verify that the shutdown result has been set
        assert!(
            shutdown_result.load(atomic::Ordering::SeqCst),
            "Stream should have finished"
        );
    }
}
