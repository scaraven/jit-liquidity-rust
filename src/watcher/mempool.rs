use std::sync::Arc;

use eyre::Result;

use alloy::{providers::Provider, pubsub::PubSubFrontend, rpc::types::Transaction};
use futures_util::StreamExt;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver},
    task::JoinHandle,
};

use crate::{shutdownconfig::ShutdownConfig, subscribefilter::ShallowFilter};

pub struct MemPool {
    provider: Arc<dyn Provider<PubSubFrontend> + Send + Sync>,
    shutdown_config: ShutdownConfig,
}

impl MemPool {
    pub fn new(
        provider: Arc<dyn Provider<PubSubFrontend> + Send + Sync>,
        shutdown_config: ShutdownConfig,
    ) -> Self {
        Self {
            provider,
            shutdown_config,
        }
    }

    // Subscribe to mempool and send transactions to buffer
    pub async fn subscribe<F>(
        &self,
        filter_type: F,
    ) -> Result<(
        JoinHandle<()>,
        UnboundedReceiver<Transaction>,
        ShutdownConfig,
    )>
    where
        F: ShallowFilter + Clone + Send + Sync + 'static,
    {
        let (sender, recv) = mpsc::unbounded_channel::<Transaction>();

        let sub = self
            .provider
            .subscribe_pending_transactions()
            .await
            .unwrap();
        let provider = self.provider.clone();

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
        // Clone items
        let shutdown_config = self.shutdown_config.clone();

        let handle = tokio::spawn(async move {
            let shutdown_config = shutdown_config;
            // Take ownership of tx
            let sender = sender;
            let mut stream = pinned_stream; // Store the stream in a pinned variable
            loop {
                tokio::select! {
                    tx = stream.next() => {
                        if shutdown_config.is_shutdown() {
                            break;
                        }
                        if let Some(tx) = tx {
                            // Send transaction to channel
                            println!("Pending transaction: {:#?}", tx);
                            // TODO: Better error handling
                            sender.send(tx).unwrap();
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        if shutdown_config.is_shutdown() {
                            break;
                        }
                    }
                }
            }
            // Gracefully shutdown channel
            drop(sender);

            shutdown_config.finish();
        });

        Ok((handle, recv, self.shutdown_config.clone()))
    }
}

#[cfg(test)]
mod tests {

    use alloy::{network::TransactionBuilder, primitives::U256, rpc::types::TransactionRequest};
    use subscribefilter::ShallowFilterType;

    use crate::{
        membuilder::{create_ws_provider, MemPoolBuilder},
        setup, subscribefilter, testconfig,
    };

    use super::*;

    #[tokio::test]
    async fn test_subscribe_to_pending() {
        let config = testconfig::TestConfig::load();

        let provider = create_ws_provider(&config.anvil_ws_endpoint).await.unwrap();

        let (http_provider, http_addr) = setup::test_setup().await;

        let mempool = MemPoolBuilder::default()
            .with_provider(provider)
            .build()
            .await
            .unwrap();

        let (handle, mut recv, shutdown) = mempool
            .subscribe(ShallowFilterType::Recipient(http_addr))
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

        // Verify that the shutdown result has not yet been set
        assert!(!handle.is_finished(), "Stream should not have finished yet");

        assert!(
            !shutdown.is_finished(),
            "Stream should not have finished yet"
        );

        // Signal shutdown
        shutdown.shutdown();

        // Wait for the stream to finish
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Verify that the shutdown result has been set
        assert!(shutdown.is_finished(), "Stream should have finished");

        // Receive transactions
        let mut count = 0;
        while recv.recv().await.is_some() {
            count += 1;
        }

        assert_eq!(count, 3, "Should have received 3 transactions");
    }
}
