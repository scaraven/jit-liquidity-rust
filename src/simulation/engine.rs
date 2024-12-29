use std::{marker::PhantomData, sync::Arc};

use alloy::{
    network::{Ethereum, Network},
    providers::{ext::TraceApi, Provider},
    rpc::types::trace::parity::{TraceResults, TraceType},
    transports::{BoxTransport, Transport},
};
use eyre::Result;

type TxBundle<N> = Vec<<N as Network>::TransactionRequest>;

struct EngineTask<P, T = BoxTransport, N = Ethereum>
where
    P: Provider<T, N> + TraceApi<N, T>,
    T: Transport + Clone,
    N: Network,
{
    provider: Arc<P>,
    bundle: TxBundle<N>,
    // Cursed!
    _marker: PhantomData<T>,
}

impl<P, T, N> EngineTask<P, T, N>
where
    P: Provider<T, N> + TraceApi<N, T>,
    T: Transport + Clone,
    N: Network,
{
    pub fn new(provider: Arc<P>, bundle: TxBundle<N>) -> Self {
        Self {
            provider,
            bundle,
            _marker: PhantomData,
        }
    }

    // Simulate transaction internal and consume the task
    pub async fn simulate(self) -> Result<Vec<TraceResults>> {
        // Build simulation block
        let trace_list = &self
            .bundle
            .into_iter()
            .map(|tx| (tx, &[TraceType::Trace][..]))
            .collect::<Vec<(<N as Network>::TransactionRequest, &[TraceType])>>()[..];

        // Simulate transaction
        let result = self.provider.trace_call_many(trace_list).await;

        result.map_err(|e| eyre::eyre!("EngineTask SIMULATION failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{
        primitives::{Address, U256},
        rpc::types::TransactionRequest,
    };

    use crate::setup;

    use super::*;

    #[tokio::test]
    pub async fn test_bundle_execute() {
        let (provider, _client) = setup::test_setup().await;
        let provider = Arc::new(provider);

        let bob = Address::from_str("0xc0ffee254729296a45a3885639AC7E10F9d54979").unwrap();
        let dan = Address::from_str("0xdeadbeef254729296a45a3885639AC7E10F9d549").unwrap();

        let bundle = vec![
            TransactionRequest::default().to(bob).value(U256::from(150)),
            TransactionRequest::default().to(dan).value(U256::from(250)),
        ];
        let task = EngineTask::new(provider.clone(), bundle).simulate().await;

        println!("{:#?}", task);
        assert!(task.is_ok(), "Error in EngineTask simulation");
        let result = task.unwrap();
        println!("{:?}", result);
    }
}
