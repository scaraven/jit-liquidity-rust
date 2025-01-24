use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    pubsub::{PubSubFrontend, Subscription},
    transports::TransportResult,
};

pub trait AlchemyProvider {
    async fn subscribe_full_pending_alchemy_transactions(
        &self,
    ) -> TransportResult<Subscription<<Ethereum as Network>::TransactionResponse>>;
}

impl AlchemyProvider for Arc<dyn Provider<PubSubFrontend> + Send + Sync> {
    async fn subscribe_full_pending_alchemy_transactions(
        &self,
    ) -> TransportResult<Subscription<<Ethereum as Network>::TransactionResponse>> {
        let id = self
            .client()
            .request("eth_subscribe", ("alchemy_pendingTransactions",))
            .await?;
        self.root().get_subscription(id).await
    }
}
