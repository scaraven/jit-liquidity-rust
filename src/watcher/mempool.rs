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
    provider: Arc<dyn Provider<PubSubFrontend>>,
    shutdown_config: ShutdownConfig,
}

impl MemPool {
    pub fn new(
        provider: Arc<dyn Provider<PubSubFrontend>>,
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

        let sub = self.provider.subscribe_full_pending_transactions().await?;

        // Filter stream based on filter type
        let stream = sub.into_stream().filter_map(move |tx| {
            let filter_type = filter_type.clone();
            async move {
                if filter_type.filter(&tx) {
                    Some(tx)
                } else {
                    None
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

    use subscribefilter::ShallowFilterType;

    use crate::{
        alchemy::AlchemyProvider,
        membuilder::{create_ws_provider, MemPoolBuilder},
        subscribefilter, testconfig,
    };

    use super::*;

    #[tokio::test]
    async fn test_subscribe_to_pending() {
        let config = testconfig::TestConfig::load();

        let provider = Arc::new(AlchemyProvider::new(
            create_ws_provider(&config.alchemy_ws_endpoint.unwrap())
                .await
                .unwrap(),
        ));

        let mempool = MemPoolBuilder::default()
            .with_provider(provider)
            .build()
            .await
            .unwrap();

        let (_handle, mut recv, shutdown) =
            mempool.subscribe(ShallowFilterType::None).await.unwrap();

        tokio::select! {
            out = recv.recv() => {
                assert!(out.is_some(), "Should have received a transaction");
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                panic!("Timed out waiting for transaction");
            }
        }

        // Shutdown the mempool
        shutdown.finish();
    }

    #[tokio::test]
    async fn test_subscribe_shutdown() {
        let config = testconfig::TestConfig::load();

        let provider = Arc::new(AlchemyProvider::new(
            create_ws_provider(&config.alchemy_ws_endpoint.unwrap())
                .await
                .unwrap(),
        ));

        let mempool = MemPoolBuilder::default()
            .with_provider(provider)
            .build()
            .await
            .unwrap();

        let (handle, _recv, shutdown) = mempool.subscribe(ShallowFilterType::None).await.unwrap();

        // Allow subscription to initialize
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

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
    }
}
