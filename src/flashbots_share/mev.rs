use std::sync::Arc;

use alloy::{
    network::EthereumWallet,
    providers::Provider,
    rpc::types::Transaction,
    transports::http::{Client, Http},
};

use crate::jit_bundler::SandwichBundler;

use eyre::Result;

struct FlashBotMev<P: Provider<Http<Client>>, S: SandwichBundler<P, Http<Client>>> {
    provider: Arc<P>,
    flashbot_provider: Arc<P>,
    wallet: EthereumWallet,
    bundler: S,
    tx: Transaction,
}

impl<P, S> FlashBotMev<P, S>
where
    P: Provider<Http<Client>>,
    S: SandwichBundler<P, Http<Client>>,
{
    pub fn new(
        provider: Arc<P>,
        flashbot_provider: Arc<P>,
        wallet: EthereumWallet,
        bundler: S,
        tx: Transaction,
    ) -> Self {
        Self {
            provider,
            flashbot_provider,
            wallet,
            bundler,
            tx,
        }
    }

    pub async fn sim_bundle(self) -> Result<()> {
        let sandwich = self.bundler.build(self.provider, self.tx.into()).await?;

        Ok(())
    }
}
