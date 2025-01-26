use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::{Provider, RootProvider},
    pubsub::{PubSubFrontend, Subscription},
    rpc::client::RpcClientInner,
    transports::{RpcError, TransportErrorKind},
};
use async_trait::async_trait;

pub struct AlchemyProvider {
    provider: Arc<dyn Provider<PubSubFrontend>>,
}

impl AlchemyProvider {
    pub fn new(provider: Arc<dyn Provider<PubSubFrontend>>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Provider<PubSubFrontend> for AlchemyProvider {
    fn client(&self) -> &RpcClientInner<PubSubFrontend> {
        self.provider.client()
    }

    fn root(&self) -> &RootProvider<PubSubFrontend> {
        self.provider.root()
    }

    async fn subscribe_full_pending_transactions(
        &self,
    ) -> Result<
        Subscription<<Ethereum as Network>::TransactionResponse>,
        RpcError<TransportErrorKind>,
    > {
        let id = self
            .client()
            .request("eth_subscribe", ("alchemy_pendingTransactions",))
            .await?;
        self.root().get_subscription(id).await
    }
}
