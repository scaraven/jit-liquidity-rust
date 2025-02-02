use std::sync::Arc;

use alloy::{
    network::EthereumWallet,
    providers::Provider,
    rpc::types::{mev::SimBundleResponse, Transaction},
    signers::Signer,
    transports::http::{Client, Http},
};

use super::sandwich_bundler::SandwichBundler;
use super::{bundle_forwarder::BundleForwarder, bundler};

use eyre::Result;

pub struct FlashBotMev<
    'a,
    P: Provider<Http<Client>>,
    B: SandwichBundler<P, Http<Client>>,
    S: Signer + Clone + Send + Sync + 'static,
> {
    provider: Arc<P>,
    flashbot_provider: Arc<P>,
    tx_wallet: &'a EthereumWallet,
    flashbot_signer: S,
    bundler: B,
    sandwich_tx: Transaction,
}

impl<'a, P, B, S> FlashBotMev<'a, P, B, S>
where
    P: Provider<Http<Client>>,
    B: SandwichBundler<P, Http<Client>>,
    S: Signer + Clone + Send + Sync + 'static,
{
    pub fn new(
        provider: Arc<P>,
        flashbot_provider: Arc<P>,
        tx_wallet: &'a EthereumWallet,
        flashbot_signer: S,
        bundler: B,
        sandwich_tx: Transaction,
    ) -> Self {
        Self {
            provider,
            flashbot_provider,
            tx_wallet,
            flashbot_signer,
            bundler,
            sandwich_tx,
        }
    }

    async fn build_forwarder(
        self,
        block_number: u64,
    ) -> Result<BundleForwarder<P, S, Http<Client>>> {
        let sandwich = self
            .bundler
            .build(self.provider, self.sandwich_tx.clone().into())
            .await?;

        let bundle = bundler::create_bundle(
            self.tx_wallet,
            sandwich.0,
            self.sandwich_tx,
            sandwich.1,
            block_number,
        )
        .await?;

        Ok(BundleForwarder::new(
            self.flashbot_provider,
            self.flashbot_signer,
            bundle,
        ))
    }

    pub async fn sim_bundle(self, block_number: u64) -> Result<SimBundleResponse> {
        let bot = self.build_forwarder(block_number).await?;

        bot.simulate_bundle().await.map_err(|e| eyre::eyre!(e))
    }

    pub async fn send_bundle(self, block_number: u64) -> Result<SimBundleResponse> {
        let bot = self.build_forwarder(block_number).await?;

        bot.send_bundle().await.map_err(|e| eyre::eyre!(e))
    }
}
