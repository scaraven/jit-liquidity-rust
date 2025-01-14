use alloy::{
    consensus::Transaction,
    network::{EthereumWallet, TransactionBuilder},
    rpc::types::{
        mev::{BundleItem, SendBundleRequest},
        Transaction as RpcTransaction, TransactionRequest,
    },
};

use eyre::Result;
use tokio_stream::StreamExt;

pub async fn create_bundle(
    wallet: &EthereumWallet,
    frontrun: Vec<TransactionRequest>,
    sandwich: RpcTransaction,
    backrun: Vec<TransactionRequest>,
    block_number: u64,
) -> Result<SendBundleRequest> {
    // Sign frontrun and backrun transaction and convert into vector of bytes
    // If any single transaction fails, the entire function will fail
    let signed_front = tokio_stream::iter(frontrun)
        .then(|tx| async {
            tx.build(wallet)
                .await
                .map(|sig| sig.input().clone())
                .map_err(eyre::Report::new)
        })
        .collect::<Result<Vec<_>>>()
        .await?;
    let signed_back = tokio_stream::iter(backrun)
        .then(|tx| async {
            tx.build(wallet)
                .await
                .map(|sig| sig.input().clone())
                .map_err(eyre::Report::new)
        })
        .collect::<Result<Vec<_>>>()
        .await?;

    // Receive the sandwich transaction and convert into bytes
    let sandwich_hash = sandwich.input().clone();

    // Create a vector of BundleItems
    let mut bundle_items: Vec<BundleItem> = Vec::new();

    for tx in signed_front {
        bundle_items.push(BundleItem::Tx {
            tx,
            can_revert: false,
        });
    }

    bundle_items.push(BundleItem::Tx {
        tx: sandwich_hash,
        can_revert: false,
    });

    for tx in signed_back {
        bundle_items.push(BundleItem::Tx {
            tx,
            can_revert: false,
        });
    }

    let bundle = SendBundleRequest::new(
        block_number,
        None,
        alloy::rpc::types::mev::ProtocolVersion::V0_1,
        bundle_items,
    );

    Ok(bundle)
}
