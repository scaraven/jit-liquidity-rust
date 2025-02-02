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
            TxEngineFilterType::AccountSlotModified(addr) => {
                tx.state.get(addr).is_some_and(|acc| acc.is_touched())
            }
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

#[cfg(test)]
mod tests {
    use alloy::hex::FromHex;
    use revm::primitives::{Account, Bytes, HashMap, Output, SuccessReason};

    use crate::utils::addresses;

    use super::*;

    fn setup() -> (Bytes, Bytes, Vec<ResultAndState>) {
        let alice = Bytes::from_hex("0xdd0d0426504F32593B4435e0A5006bfA7b187ef8").unwrap();
        let zero = Bytes::from_hex("0x00").unwrap();

        let txs = vec![
            ResultAndState {
                state: Default::default(),
                result: ExecutionResult::Success {
                    output: Output::Call(alice.clone()),
                    gas_used: 0,
                    reason: SuccessReason::Return,
                    logs: Vec::new(),
                    gas_refunded: 0,
                },
            },
            ResultAndState {
                state: Default::default(),
                result: ExecutionResult::Revert {
                    output: zero.clone(),
                    gas_used: 0,
                },
            },
        ];

        (alice, zero, txs)
    }

    #[test]
    fn test_success_filter() {
        let (_alice, _zero, txs) = setup();

        let filter = RootFilter {
            filter: TxEngineFilterType::IsSuccess,
        };

        let result = filter.filter(txs);
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0].result, ExecutionResult::Success { .. }));
    }

    #[test]
    fn test_identity_filter() {
        let (_alice, _zero, txs) = setup();

        let filter = RootFilter {
            filter: TxEngineFilterType::Identity,
        };

        let result = filter.filter(txs.clone());
        assert_eq!(result.len(), 2);
        assert_eq!(result, txs);
    }

    #[test]
    fn test_revert_filter() {
        let (_alice, _zero, txs) = setup();

        let filter = RootFilter {
            filter: TxEngineFilterType::Reverted,
        };

        let result = filter.filter(txs.clone());
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0].result, ExecutionResult::Revert { .. }));
    }

    #[test]
    fn test_account_slot_modified_filter() {
        let alice_bytes = Bytes::from_hex("0xE0106B0a4ead5A806e9059BD0EB4Acfad15E9740").unwrap();
        let alice = Address::from_slice(alice_bytes.as_ref());
        let bob = addresses::get_address("0x97e750a788C14b62b9d8b84ED2c10b912EDf52F9").unwrap();

        let zero = Bytes::from_hex("0x00").unwrap();

        let mut alice_account = Account::new_not_existing();
        alice_account.mark_touch();

        let mut bob_account = Account::new_not_existing();
        bob_account.mark_touch();

        // Create a state where the storage slot of alice is modified
        let mut state_alice = HashMap::default();
        state_alice.insert(alice, alice_account);

        let mut state_bob = HashMap::default();
        state_bob.insert(bob, bob_account);

        let txs = vec![
            ResultAndState {
                state: state_alice,
                result: ExecutionResult::Success {
                    output: Output::Call(alice_bytes.clone()),
                    gas_used: 0,
                    reason: SuccessReason::Return,
                    logs: Vec::new(),
                    gas_refunded: 0,
                },
            },
            ResultAndState {
                state: state_bob,
                result: ExecutionResult::Revert {
                    output: zero,
                    gas_used: 0,
                },
            },
        ];

        let filter = RootFilter {
            filter: TxEngineFilterType::AccountSlotModified(alice),
        };

        let result = filter.filter(txs.clone());
        assert_eq!(result.len(), 1);
        // Assert we have recovered the first account
        assert!(matches!(result[0].result, ExecutionResult::Success { .. }));
    }

    #[test]
    fn test_and_filter() {
        let (alice_bytes, zero, txs) = setup();
        let alice = Address::from_slice(alice_bytes.as_ref());
        let bob = addresses::get_address("0x97e750a788C14b62b9d8b84ED2c10b912EDf52F9").unwrap();

        let filter = AndFilter {
            inner: RootFilter {
                filter: TxEngineFilterType::Identity,
            },
            filters: vec![TxEngineFilterType::IsSuccess, TxEngineFilterType::Reverted],
        };

        let result = filter.filter(txs);
        assert_eq!(result.len(), 0);

        let mut alice_account = Account::new_not_existing();
        alice_account.mark_touch();

        let mut bob_account = Account::new_not_existing();
        bob_account.mark_touch();

        // Create a state where the storage slot of alice is modified
        let mut state_alice = HashMap::default();
        state_alice.insert(alice, alice_account);

        let mut state_bob = HashMap::default();
        state_bob.insert(bob, bob_account);

        let txs = vec![
            ResultAndState {
                state: state_alice,
                result: ExecutionResult::Success {
                    output: Output::Call(alice_bytes.clone()),
                    gas_used: 0,
                    reason: SuccessReason::Return,
                    logs: Vec::new(),
                    gas_refunded: 0,
                },
            },
            ResultAndState {
                state: state_bob,
                result: ExecutionResult::Revert {
                    output: zero,
                    gas_used: 0,
                },
            },
        ];

        let filter = AndFilter {
            inner: RootFilter {
                filter: TxEngineFilterType::Identity,
            },
            filters: vec![
                TxEngineFilterType::IsSuccess,
                TxEngineFilterType::AccountSlotModified(alice),
            ],
        };

        let result2 = filter.filter(txs);
        assert_eq!(result2.len(), 1);
        assert!(matches!(
            &result2[0].result,
            ExecutionResult::Success { .. }
        ));
    }

    #[test]
    fn test_or_filter() {
        let (alice_bytes, zero, txs) = setup();
        let alice = Address::from_slice(alice_bytes.as_ref());
        let bob = addresses::get_address("0x97e750a788C14b62b9d8b84ED2c10b912EDf52F9").unwrap();

        let filter = OrFilter {
            inner: RootFilter {
                filter: TxEngineFilterType::Identity,
            },
            filters: vec![TxEngineFilterType::IsSuccess, TxEngineFilterType::Reverted],
        };

        let result = filter.filter(txs.clone());
        assert_eq!(result.len(), 2);
        assert_eq!(result, txs);

        let mut alice_account = Account::new_not_existing();
        alice_account.mark_touch();

        let mut bob_account = Account::new_not_existing();
        bob_account.mark_touch();

        // Create a state where the storage slot of alice is modified
        let mut state_alice = HashMap::default();
        state_alice.insert(alice, alice_account);

        let mut state_bob = HashMap::default();
        state_bob.insert(bob, bob_account);

        let txs = vec![
            ResultAndState {
                state: state_alice,
                result: ExecutionResult::Success {
                    output: Output::Call(alice_bytes.clone()),
                    gas_used: 0,
                    reason: SuccessReason::Return,
                    logs: Vec::new(),
                    gas_refunded: 0,
                },
            },
            ResultAndState {
                state: state_bob,
                result: ExecutionResult::Revert {
                    output: zero,
                    gas_used: 0,
                },
            },
        ];

        let filter = OrFilter {
            inner: RootFilter {
                filter: TxEngineFilterType::Identity,
            },
            filters: vec![
                TxEngineFilterType::IsSuccess,
                TxEngineFilterType::AccountSlotModified(alice),
            ],
        };

        let result2 = filter.filter(txs);
        assert_eq!(result2.len(), 1);
        assert!(matches!(
            &result2[0].result,
            ExecutionResult::Success { .. }
        ));
    }
}
