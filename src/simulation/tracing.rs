use eyre::Result;
use std::sync::Arc;

use alloy::{
    eips::BlockId, network::Network, providers::Provider, rpc::types::TransactionRequest,
    transports::Transport,
};
use revm::{
    db::{AlloyDB, CacheDB},
    primitives::{ExecutionResult, Output, ResultAndState},
    DatabaseCommit, Evm,
};

/// Internal function to execute a transaction with revm.
///
/// # Arguments
///
/// * `cache_db` - The cache database.
/// * `desired_tx` - The desired transaction.
/// * `commit` - Whether to commit the changes to the database.
///
/// # Returns
///
/// * `Result<ResultAndState>` - The result and state of the execution.
fn revm_call_internal<T, N, P>(
    cache_db: &mut CacheDB<AlloyDB<T, N, Arc<P>>>,
    desired_tx: TransactionRequest,
    commit: bool,
) -> Result<ResultAndState>
where
    T: Transport + Clone,
    N: Network,
    P: Provider<T, N>,
{
    // Build a new evm instance with desired tx
    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = desired_tx.from.unwrap_or(tx.caller);
            tx.value = desired_tx.value.unwrap_or(tx.value);
            tx.gas_limit = desired_tx.gas.unwrap_or(tx.gas_limit);
            tx.nonce = desired_tx.nonce;
            tx.data = desired_tx.input.input.unwrap_or(tx.data.clone());
            tx.transact_to = desired_tx.to.unwrap_or(tx.transact_to);
        })
        .build();

    // Execute the transaction and view results
    let ref_tx = evm
        .transact()
        .map_err(|e| eyre::eyre!("Transaction failed: {:?}", e))?;
    let result = ref_tx.result.clone();

    let ret = match result {
        ExecutionResult::Success {
            output: Output::Call(_value),
            ..
        } => ref_tx,
        result => {
            return Err(eyre::eyre!("Execution failed: {:?}", result));
        }
    };

    // Commit change to db if we have to
    if commit {
        evm.db_mut().commit(ret.state.clone());
    }

    Ok(ret)
}

pub fn revm_call_write<T, N, P>(
    cache_db: &mut CacheDB<AlloyDB<T, N, Arc<P>>>,
    desired_tx: TransactionRequest,
) -> Result<ResultAndState>
where
    T: Transport + Clone,
    N: Network,
    P: Provider<T, N>,
{
    revm_call_internal(cache_db, desired_tx, true)
}

pub fn revm_call_read_only<T, N, P>(
    cache_db: &mut CacheDB<AlloyDB<T, N, Arc<P>>>,
    desired_tx: TransactionRequest,
) -> Result<ResultAndState>
where
    T: Transport + Clone,
    N: Network,
    P: Provider<T, N>,
{
    revm_call_internal(cache_db, desired_tx, false)
}

pub fn init_cache_db<T: Transport + Clone, N: Network, P: Provider<T, N>>(
    provider: Arc<P>,
    block: BlockId,
) -> CacheDB<AlloyDB<T, N, Arc<P>>> {
    CacheDB::new(AlloyDB::new(provider, block).unwrap())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{
        eips::BlockNumberOrTag,
        primitives::{Address, FixedBytes, U256},
    };

    use crate::{
        interfaces::erc20,
        utils::{addresses, blockchain_utils, setup},
    };

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_revm_call_read_only() {
        // setup code
        let (provider, client) = setup::test_setup().await;
        let provider = Arc::new(provider);

        let mut cache_db = init_cache_db(provider.clone(), BlockNumberOrTag::Latest.into());

        const VALUE: i32 = 150;
        let bob = Address::from_str("0x098a1A2009184D4D24E57F4bD58C144E8C037933").unwrap();

        // Send 150 wei to bob
        let desired_tx = TransactionRequest::default()
            .to(bob)
            .from(client)
            .value(U256::from(VALUE));

        // Execute with revm, do not commit to the database
        let output = revm_call_read_only(&mut cache_db, desired_tx);
        assert!(output.is_ok());
        let output = output.unwrap();

        println!("{:?}", output);
        println!("{:?}", &client);
        // Check that stage changes are as expected
        assert!(output.state.contains_key(&bob));
        assert!(output.state.contains_key(&client));
        assert_eq!(
            output.state.get(&bob).unwrap().info.balance,
            U256::from(VALUE)
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_revm_call_write() {
        let (provider, client) = setup::test_setup().await;
        let provider = Arc::new(provider);

        let mut cache_db = init_cache_db(provider.clone(), BlockNumberOrTag::Latest.into());

        const VALUE: i32 = 150;
        let bob = Address::from_str("0x97e750a788C14b62b9d8b84ED2c10b912EDf52F9").unwrap();

        // Send 150 wei to bob
        let tx1 = TransactionRequest::default()
            .to(bob)
            .from(client)
            .value(U256::from(VALUE));

        // Send 300 wei to bob
        let tx2 = TransactionRequest::default()
            .to(bob)
            .from(client)
            .value(U256::from(VALUE * 2));

        // Execute with revm, commit to the database
        let first = revm_call_write(&mut cache_db, tx1);
        assert!(first.is_ok());

        let first = first.unwrap();

        assert!(first.state.contains_key(&bob));
        assert!(first.state.contains_key(&client));
        assert_eq!(
            first.state.get(&bob).unwrap().info.balance,
            U256::from(VALUE)
        );

        // Execute with revm, commit to the database
        let second = revm_call_write(&mut cache_db, tx2);
        assert!(second.is_ok());

        let second = second.unwrap();

        println!("{:?}", second);
        println!("{:p}", &client);
        assert!(second.state.contains_key(&bob));
        assert!(second.state.contains_key(&client));
        // Expect bob to have 450 wei
        assert_eq!(
            second.state.get(&bob).unwrap().info.balance,
            U256::from(3 * VALUE)
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_revm_erc20_call() {
        let (provider, _client) = setup::test_setup().await;
        let provider = Arc::new(provider);

        let mut cache_db = init_cache_db(provider.clone(), BlockNumberOrTag::Latest.into());

        const VALUE: i32 = 150;
        let bob = addresses::get_address("0x7E219AAf9339eA8f08c381632DEe3CeC94AA4054").unwrap();
        let weth = *addresses::WETH;

        const BALANCE_SLOT: u8 = 3;
        let storage_slot = blockchain_utils::calculate_slot_mapping(
            FixedBytes::<32>::left_padding_from(&bob.into_array()).to_vec(),
            BALANCE_SLOT,
        );
        println!("{:?}", storage_slot);

        // Create an erc20 transaction request and execute it with revm
        let tx = erc20::transfer(&provider, weth, bob, U256::from(VALUE));

        let result = revm_call_read_only(&mut cache_db, tx);
        assert!(result.is_ok());

        let log = result.unwrap();

        println!("{:?}", log.state);

        assert!(log.state.contains_key(&weth));
        assert!(log
            .state
            .get(&weth)
            .unwrap()
            .storage
            .contains_key(&storage_slot));

        let slot_changed = log
            .state
            .get(&weth)
            .unwrap()
            .storage
            .get(&storage_slot)
            .unwrap();
        assert_eq!(slot_changed.original_value, U256::ZERO);
        assert_eq!(slot_changed.present_value, U256::from(VALUE));
    }
}
