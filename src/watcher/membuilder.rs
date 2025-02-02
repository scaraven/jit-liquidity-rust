use std::sync::Arc;

use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    pubsub::PubSubFrontend,
};
use eyre::Result;

use super::{mempool::MemPool, shutdownconfig::ShutdownConfig};

#[derive(Default)]
pub struct MemPoolBuilder {
    provider: Option<Arc<dyn Provider<PubSubFrontend>>>,
    shutdown_config: ShutdownConfig,
}

impl MemPoolBuilder {
    /// Set the provider for the mempool.
    pub fn with_provider(mut self, provider: Arc<dyn Provider<PubSubFrontend>>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the shutdown configuration for the mempool.
    pub fn with_shutdown_config(mut self, shutdown_config: ShutdownConfig) -> Self {
        self.shutdown_config = shutdown_config;
        self
    }

    /// Build the mempool instance.
    pub async fn build(self) -> Result<MemPool> {
        Ok(MemPool::new(
            self.provider
                .ok_or_else(|| eyre::eyre!("Provider not set"))?,
            self.shutdown_config,
        ))
    }
}

pub async fn create_ws_provider(rpc_url: &str) -> Result<Arc<dyn Provider<PubSubFrontend>>> {
    let provider = ProviderBuilder::new()
        .on_ws(WsConnect::new(rpc_url))
        .await?;
    Ok(Arc::new(provider))
}
