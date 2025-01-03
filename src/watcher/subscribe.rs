use std::sync::Arc;

use crate::subscribe_filter::ShallowFilter;
use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::Transaction,
};
use eyre::Result;
use futures_util::StreamExt;
use tokio::{sync::Mutex, task::JoinHandle};

pub async fn subscribe_to_pending<P, T>(
    provider: Arc<P>,
    filter_type: T,
    tx_buffer: Arc<Mutex<Vec<Transaction>>>,
) -> Result<JoinHandle<()>>
where
    P: Provider<PubSubFrontend> + Send + Sync + 'static,
    T: ShallowFilter + Clone + Send + Sync + 'static,
{
    let sub = provider.subscribe_pending_transactions().await.unwrap();

    // Filter stream based on filter type
    let stream = sub
        .into_stream()
        // Filter and map transaction hashes based on whether we find valid corresponding transactions
        .filter_map(move |tx_hash| {
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
        })
        .take(1);
    println!("Awaiting pending transactions...");

    // Take the stream and print the pending transaction.
    let handle = tokio::spawn(async move {
        let mut stream = Box::pin(stream);
        while let Some(tx) = stream.as_mut().next().await {
            // Add transaction to buffer
            println!("Pending transaction: {:#?}", tx);
            let mut buffer = tx_buffer.lock().await;
            buffer.push(tx);
        }
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

        // Start monitoring pending transactions
        let handle = subscribe_to_pending(
            provider.clone(),
            ShallowFilterType::Recipient(http_addr),
            tx_buffer.clone(),
        )
        .await
        .unwrap();

        // Send a transaction to ourselves
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

        assert!(handle.await.is_ok(), "Error in pending transaction stream");

        // acquire mutex after stream has finished
        let buffer = tx_buffer.lock().await;
        assert!(buffer.len() == 1, "No transactions received");
    }
}
