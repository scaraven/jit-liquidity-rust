use std::{marker::PhantomData, str::FromStr, sync::Arc};

use alloy::{
    network::{Ethereum, Network},
    primitives::{Address, U256},
    providers::{ext::DebugApi, Provider},
    rpc::types::{
        trace::{
            geth::{GethDebugTracingCallOptions, GethTrace},
            parity::{TraceResults, TraceType},
        },
        Bundle, StateContext, TransactionRequest,
    },
    transports::{BoxTransport, Transport},
};
use eyre::Result;

type TxBundle<N> = Vec<<N as Network>::TransactionRequest>;

struct EngineTask<P, T = BoxTransport, N = Ethereum>
where
    P: Provider<T, N> + DebugApi<N, T>,
    T: Transport + Clone,
    N: Network,
{
    provider: Arc<P>,
    bundle: <N as Network>::TransactionRequest,
    // Cursed!
    _marker: PhantomData<T>,
}

impl<P, T, N> EngineTask<P, T, N>
where
    P: Provider<T, N> + DebugApi<N, T>,
    T: Transport + Clone,
    N: Network,
{
    pub fn new(provider: Arc<P>, bundle: <N as Network>::TransactionRequest) -> Self {
        Self {
            provider,
            bundle,
            _marker: PhantomData,
        }
    }

    // Simulate transaction with trace_callMany and consume the task
    pub async fn simulate(self) -> Result<Vec<GethTrace>> {
        // Build simulation block
        /* let trace_list = &self
            .bundle
            .into_iter()
            .map(|tx| (tx, &[TraceType::Trace][..]))
            .collect::<Vec<(<N as Network>::TransactionRequest, &[TraceType])>>()[..];
        */

        let bob = Address::from_str("0xc0ffee254729296a45a3885639AC7E10F9d54979").unwrap();
        let dan = Address::from_str("0xdeadbeef254729296a45a3885639AC7E10F9d549").unwrap();
        // Define transactions.
        let tx1 = TransactionRequest::default().to(bob).value(U256::from(150));
        let tx2 = TransactionRequest::default().to(dan).value(U256::from(150));

        // Simulate transaction
        let bundles = vec![Bundle {
            transactions: vec![tx1, tx2],
            block_override: None,
        }];

        // Define the state context and trace option.
        let state_context = StateContext::default();
        let trace_options = GethDebugTracingCallOptions::default();

        // Call `debug_trace_call_many` on the provider.
        let result = self
            .provider
            .debug_trace_call_many(bundles, state_context, trace_options)
            .await;

        // let result = self.provider.debug_trace_call_many(&self.bundle, &[TraceType::Trace]).await;

        result.map_err(|e| eyre::eyre!("EngineTask SIMULATIOsN failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{
        primitives::{Address, U256},
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
        transports::http::reqwest::Url,
    };

    use crate::setup;

    use super::*;

    #[tokio::test]
    pub async fn test_bundle_execute() {
        // Use reth node for testing
        let (provider, _client) = setup::test_setup().await;
        let provider = Arc::new(provider);

        let bob = Address::from_str("0xc0ffee254729296a45a3885639AC7E10F9d54979").unwrap();
        let dan = Address::from_str("0xdeadbeef254729296a45a3885639AC7E10F9d549").unwrap();

        let tx = TransactionRequest::default().to(bob).value(U256::from(100));

        let _bundle = vec![
            TransactionRequest::default().to(bob).value(U256::from(150)),
            TransactionRequest::default().to(dan).value(U256::from(250)),
        ];
        let task = EngineTask::new(provider.clone(), tx).simulate().await;

        println!("{:#?}", task);
        assert!(task.is_ok(), "Error in EngineTask simulation");
        let result = task.unwrap();
        println!("{:?}", result);
    }
}
