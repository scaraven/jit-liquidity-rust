use revm::primitives::{Address, ExecutionResult, ResultAndState};

trait EngineFilter {
    fn filter(&self, tx: Vec<ResultAndState>) -> Vec<ResultAndState>;
}

enum TxEngineFilterType {
    AccountSlotModified(Address),
    Identity,
    IsSuccess,
    Reverted,
}

impl TxEngineFilterType {
    fn filter(&self, tx: &ResultAndState) -> bool {
        match self {
            TxEngineFilterType::AccountSlotModified(addr) => tx.state.contains_key(addr),
            TxEngineFilterType::Identity => true,
            TxEngineFilterType::IsSuccess => tx.result.is_success(),
            TxEngineFilterType::Reverted => matches!(&tx.result, ExecutionResult::Revert { .. }),
        }
    }
}

struct RootFilter {
    filter: TxEngineFilterType,
}

struct OrFilter<T: EngineFilter> {
    inner: T,
    filters: Vec<TxEngineFilterType>,
}

struct AndFilter<T: EngineFilter> {
    inner: T,
    filters: Vec<TxEngineFilterType>,
}

impl EngineFilter for RootFilter {
    fn filter(&self, txs: Vec<ResultAndState>) -> Vec<ResultAndState> {
        txs.into_iter()
            .filter(|tx| self.filter.filter(tx))
            .collect()
    }
}

impl<T: EngineFilter> EngineFilter for OrFilter<T> {
    fn filter(&self, txs: Vec<ResultAndState>) -> Vec<ResultAndState> {
        let input = txs
            .into_iter()
            .filter(|tx| self.filters.iter().any(|f| f.filter(tx)))
            .collect();
        self.inner.filter(input)
    }
}

impl<T: EngineFilter> EngineFilter for AndFilter<T> {
    fn filter(&self, txs: Vec<ResultAndState>) -> Vec<ResultAndState> {
        let input = txs
            .into_iter()
            .filter(|tx| self.filters.iter().all(|f| f.filter(tx)))
            .collect();
        self.inner.filter(input)
    }
}
