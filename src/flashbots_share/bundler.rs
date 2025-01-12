use alloy::{
    consensus::Transaction,
    network::{EthereumWallet, TransactionBuilder},
    rpc::types::{
        mev::{BundleItem, SendBundleRequest},
        Transaction as RpcTransaction, TransactionRequest,
    },
};

use eyre::Result;

pub async fn create_bundle(
    wallet: EthereumWallet,
    frontrun: TransactionRequest,
    sandwich: RpcTransaction,
    backrun: TransactionRequest,
    block_number: u64,
) -> Result<SendBundleRequest> {
    // Sign frontrun and backrun transaction and convert into bytes
    let signed_front = frontrun.build(&wallet).await?.input().clone();
    let signed_back = backrun.build(&wallet).await?.input().clone();

    // Receive the sandwich transaction and convert into bytes
    let sandwich_hash = sandwich.input().clone();

    // Create Bundle Items
    let bundle_items = vec![
        BundleItem::Tx {
            tx: signed_front,
            can_revert: false,
        },
        BundleItem::Tx {
            tx: sandwich_hash,
            can_revert: false,
        },
        BundleItem::Tx {
            tx: signed_back,
            can_revert: false,
        },
    ];

    let bundle = SendBundleRequest::new(
        block_number,
        None,
        alloy::rpc::types::mev::ProtocolVersion::V0_1,
        bundle_items,
    );

    Ok(bundle)
}
