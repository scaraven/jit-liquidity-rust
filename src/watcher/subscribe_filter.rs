use alloy::consensus::TxType;
use alloy::primitives::{Address, Bytes, TxKind};
use alloy::rpc::types::Transaction;

pub trait ShallowFilter {
    fn filter(&self, tx: &Transaction) -> bool;
}

#[derive(Clone, Debug)]
pub enum ShallowFilterType {
    Recipient(Address),
    CallData(Bytes),
    None,
}

impl ShallowFilter for ShallowFilterType {
    fn filter(&self, tx: &Transaction) -> bool {
        match self {
            ShallowFilterType::Recipient(addr) => filter_by_addr(tx, addr),
            ShallowFilterType::None => true,
            ShallowFilterType::CallData(data) => filter_by_data(tx, data),
        }
    }
}

fn filter_by_addr(tx: &Transaction, expected: &Address) -> bool {
    match tx.inner.tx_type() {
        TxType::Eip1559 => {
            let tx = tx.inner.as_eip1559().unwrap();
            let to = tx.clone().strip_signature().to;
            match to {
                TxKind::Call(addr) => addr == *expected,
                TxKind::Create => false,
            }
        }
        _ => false,
    }
}

fn filter_by_data(tx: &Transaction, expected: &Bytes) -> bool {
    match tx.inner.tx_type() {
        TxType::Eip1559 => {
            let tx = tx.inner.as_eip1559().unwrap();
            let data = tx.clone().strip_signature().input;
            data == *expected
        }
        _ => false,
    }
}
