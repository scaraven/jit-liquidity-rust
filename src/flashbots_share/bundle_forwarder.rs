use std::{marker::PhantomData, sync::Arc};

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::{
        client::RpcCall,
        types::{
            mev::{SendBundleRequest, SimBundleResponse},
            TransactionRequest,
        },
    },
    signers::Signer,
    transports::{
        http::{reqwest, Http},
        BoxTransport, Transport, TransportResult,
    },
};

use alloy_mev::MevHttp;

struct BundleForwarder<
    P: Provider<T, N>,
    S: Signer + Clone + Send + Sync + 'static,
    T: Clone + Transport = BoxTransport,
    N: Network = Ethereum,
> {
    flashbot_provider: Arc<P>,
    signer: S,
    bundle: SendBundleRequest,
    _marker: PhantomData<(T, N)>,
}

impl<P, N, S> BundleForwarder<P, S, Http<reqwest::Client>, N>
where
    P: Provider<Http<reqwest::Client>, N>,
    S: Signer + Clone + Send + Sync + 'static,
    N: Network<TransactionRequest = TransactionRequest>,
{
    pub fn new(flashbot_provider: Arc<P>, signer: S, bundle: SendBundleRequest) -> Self {
        Self {
            flashbot_provider,
            signer,
            bundle,
            _marker: PhantomData,
        }
    }

    pub async fn simulate_bundle(self, signer: S) -> TransportResult<SimBundleResponse> {
        // Send the bundle to the flashbots relay
        let request = self
            .flashbot_provider
            .client()
            .make_request("mev_simBundle", (self.bundle,));

        RpcCall::new(
            request,
            MevHttp::flashbots(self.flashbot_provider.client().transport().clone(), signer),
        )
        .await
    }

    pub async fn send_bundle(self, signer: S) -> TransportResult<SimBundleResponse> {
        // Send the bundle to the flashbots relay
        let request = self
            .flashbot_provider
            .client()
            .make_request("mev_sendBundle", (self.bundle,));

        RpcCall::new(
            request,
            MevHttp::flashbots(self.flashbot_provider.client().transport().clone(), signer),
        )
        .await
    }
}
