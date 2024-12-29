use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::{
        simulate::{SimBlock, SimulatePayload, SimulatedBlock},
        TransactionRequest,
    },
    transports::{BoxTransport, Transport},
};
use eyre::Result;

type TxBundle = Vec<TransactionRequest>;

struct EngineTask<T: Transport = BoxTransport, N = Ethereum> {
    provider: Arc<dyn Provider<T, N>>,
    bundle: TxBundle,
}

impl<T: Transport + Clone, N: Network> EngineTask<T, N> {
    pub fn new(provider: Arc<dyn Provider<T, N>>, bundle: TxBundle) -> Self {
        Self { provider, bundle }
    }

    // Simulate transaction internal and consume the task
    pub async fn simulate(self) -> Result<SimulatedBlock<<N as Network>::BlockResponse>> {
        // Build transaction block
        let sim_block = SimBlock::default().extend_calls(self.bundle);

        // Simulate transaction
        let payload = SimulatePayload::default()
            .extend(sim_block)
            .with_full_transactions();

        // Send payload to provider
        let rpcblock = self.provider.simulate(&payload).await;

        match rpcblock {
            Ok(blocks) => Ok(blocks[0].clone()),
            Err(e) => Err(eyre::eyre!("EngineTask SIMULATION failed: {}", e)),
        }
    }
}
