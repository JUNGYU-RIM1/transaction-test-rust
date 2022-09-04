pub mod domain {
    use std::collections::{hash_map::Iter, HashMap, HashSet};

    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    pub enum Transaction {
        Deposit { amount: Decimal },
        Withdrawal { amount: Decimal },
        Dispute,
        Resolve,
        Chargeback,
    }

    #[derive(Debug, PartialEq)]
    pub enum TransactionState {
        Resolve,
        Dispute,
        Chargeback,
    }

    #[derive(Debug, PartialEq)]
    pub enum TransactionActionState {
        Deposit { amount: Decimal },
        Withdrawal { amount: Decimal },
    }

    #[derive(Debug, PartialEq)]
    pub struct TransactionLog {
        pub amount: TransactionActionState,
        pub state: TransactionState,
    }

    pub struct Accounts {
        user_accounts: HashMap<u16, UserAccount>,
        transaction_ids: HashSet<u32>,
    }

    impl Accounts {
        pub fn new() -> Accounts {
            Accounts {
                user_accounts: HashMap::new(),
                transaction_ids: HashSet::new(),
            }
        }

        pub fn get_user_accounts(&self) -> Iter<u16, UserAccount> {
            self.user_accounts.iter()
        }

        pub fn get_user_account(&self, client: u16) -> Option<&UserAccount> {
            self.user_accounts.get(&client)
        }

        pub fn add_transaction(&mut self, client: u16, tx: u32, transaction: Transaction) {
            if (matches!(transaction, Transaction::Deposit { amount: _ })
                || matches!(transaction, Transaction::Withdrawal { amount: _ }))
                && !self.transaction_ids.insert(tx)
            {
                return;
            }

            if let Some(x) = self.user_accounts.get_mut(&client) {
                x.change_account_state(tx, transaction);
            } else if let Some(account) = UserAccount::new(tx, transaction) {
                self.user_accounts.insert(client, account);
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct UserAccount {
        pub available: Decimal,
        pub held: Decimal,
        pub locked: bool,
        pub transaction_log: HashMap<u32, TransactionLog>,
    }

    impl UserAccount {
        fn new(tx: u32, transaction: Transaction) -> Option<UserAccount> {
            match transaction {
                Transaction::Deposit { amount } => Option::Some(UserAccount {
                    available: amount,
                    held: dec!(0),
                    locked: false,
                    transaction_log: HashMap::from([(
                        tx,
                        TransactionLog {
                            amount: TransactionActionState::Deposit { amount: amount },
                            state: TransactionState::Resolve,
                        },
                    )]),
                }),
                _ => Option::None,
            }
        }

        fn change_account_state(&mut self, tx: u32, transaction: Transaction) {
            if self.locked {
                return;
            }
            match transaction {
                Transaction::Deposit { amount } => {
                    self.transaction_log.insert(
                        tx,
                        TransactionLog {
                            amount: TransactionActionState::Deposit { amount: amount },
                            state: TransactionState::Resolve,
                        },
                    );
                    self.available = self.available + amount;
                }

                Transaction::Dispute => {
                    if let Some(x) = self.transaction_log.get_mut(&tx) {
                        if matches!(x.state, TransactionState::Resolve) {
                            match x.amount {
                                TransactionActionState::Deposit { amount } => {
                                    *x = TransactionLog {
                                        amount: TransactionActionState::Deposit { amount: amount },
                                        state: TransactionState::Dispute,
                                    };
                                    self.available = self.available - amount;
                                    self.held = self.held + amount;
                                }
                                TransactionActionState::Withdrawal { amount } => {
                                    *x = TransactionLog {
                                        amount: TransactionActionState::Withdrawal {
                                            amount: amount,
                                        },
                                        state: TransactionState::Dispute,
                                    };
                                    self.held = self.held + amount;
                                }
                            }
                        }
                    }
                }

                Transaction::Resolve => {
                    if let Some(x) = self.transaction_log.get_mut(&tx) {
                        if matches!(x.state, TransactionState::Dispute) {
                            match x.amount {
                                TransactionActionState::Deposit { amount } => {
                                    *x = TransactionLog {
                                        amount: TransactionActionState::Deposit { amount: amount },
                                        state: TransactionState::Resolve,
                                    };
                                    self.available = self.available + amount;
                                    self.held = self.held - amount;
                                }
                                TransactionActionState::Withdrawal { amount } => {
                                    *x = TransactionLog {
                                        amount: TransactionActionState::Withdrawal {
                                            amount: amount,
                                        },
                                        state: TransactionState::Resolve,
                                    };
                                    self.held = self.held - amount;
                                }
                            }
                        }
                    }
                }

                Transaction::Chargeback => {
                    if let Some(x) = self.transaction_log.get_mut(&tx) {
                        if matches!(x.state, TransactionState::Dispute) {
                            match x.amount {
                                TransactionActionState::Deposit { amount } => {
                                    *x = TransactionLog {
                                        amount: TransactionActionState::Deposit { amount: amount },
                                        state: TransactionState::Chargeback,
                                    };
                                    self.held = self.held - amount;
                                    self.locked = true;
                                }
                                TransactionActionState::Withdrawal { amount } => {
                                    *x = TransactionLog {
                                        amount: TransactionActionState::Withdrawal {
                                            amount: amount,
                                        },
                                        state: TransactionState::Chargeback,
                                    };
                                    self.held = self.held - amount;
                                    self.locked = true;
                                }
                            }
                        }
                    }
                }

                Transaction::Withdrawal { amount } => {
                    self.withdrawal(amount, tx);
                }
            }
        }

        fn withdrawal(&mut self, amount: Decimal, tx: u32) {
            if self.available >= amount {
                self.transaction_log.insert(
                    tx,
                    TransactionLog {
                        amount: TransactionActionState::Withdrawal { amount: amount },
                        state: TransactionState::Resolve,
                    },
                );
                self.available = self.available - amount;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rust_decimal_macros::dec;

    use crate::domain::{
        Accounts, Transaction, TransactionActionState, TransactionLog, TransactionState,
        UserAccount,
    };

    #[test]
    fn first_transaction_should_be_added_only_if_transaction_state_is_deposit() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(2, 2, Transaction::Deposit { amount: dec!(1000) });
        accounts.add_transaction(3, 3, Transaction::Withdrawal { amount: dec!(1000) });
        accounts.add_transaction(4, 4, Transaction::Dispute );
        accounts.add_transaction(5, 5, Transaction::Chargeback );
        accounts.add_transaction(6, 6, Transaction::Resolve );

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(100),
                held: dec!(0),
                locked: false,
                transaction_log: HashMap::from([(
                    1,
                    TransactionLog {
                        amount: TransactionActionState::Deposit { amount: dec!(100) },
                        state: TransactionState::Resolve,
                    },
                )]),
            })
        );
        assert_eq!(
            accounts.get_user_account(2),
            Some(&UserAccount {
                available: dec!(1000),
                held: dec!(0),
                locked: false,
                transaction_log: HashMap::from([(
                    2,
                    TransactionLog {
                        amount: TransactionActionState::Deposit { amount: dec!(1000) },
                        state: TransactionState::Resolve,
                    },
                )]),
            })
        );
    }

    #[test]
    fn deposit_and_withdrawal_should_be_ignored_if_same_transaction_id_is_already_existed() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Withdrawal { amount: dec!(400) });

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(100),
                held: dec!(0),
                locked: false,
                transaction_log: HashMap::from([(
                    1,
                    TransactionLog {
                        amount: TransactionActionState::Deposit { amount: dec!(100) },
                        state: TransactionState::Resolve,
                    },
                ),]),
            })
        );
    }

    #[test]
    fn money_should_be_withdrawal_if_current_amount_is_bigger_than_withdrawal_amount() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(1000) });
        accounts.add_transaction(1, 2, Transaction::Deposit { amount: dec!(1000) });
        accounts.add_transaction(1, 3, Transaction::Withdrawal { amount: dec!(1500) });

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(500),
                held: dec!(0),
                locked: false,
                transaction_log: HashMap::from([
                    (
                        1,
                        TransactionLog {
                            amount: TransactionActionState::Deposit { amount: dec!(1000) },
                            state: TransactionState::Resolve,
                        },
                    ),
                    (
                        2,
                        TransactionLog {
                            amount: TransactionActionState::Deposit { amount: dec!(1000) },
                            state: TransactionState::Resolve,
                        },
                    ),
                    (
                        3,
                        TransactionLog {
                            amount: TransactionActionState::Withdrawal { amount: dec!(1500) },
                            state: TransactionState::Resolve,
                        },
                    )
                ]),
            })
        );
    }

    #[test]
    fn money_should_not_be_withdrawal_if_current_amount_is_less_than_withdrawal_amount() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(1000) });
        accounts.add_transaction(1, 2, Transaction::Deposit { amount: dec!(1000) });
        accounts.add_transaction(1, 2, Transaction::Dispute);
        accounts.add_transaction(1, 3, Transaction::Withdrawal { amount: dec!(1500) });

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(1000),
                held: dec!(1000),
                locked: false,
                transaction_log: HashMap::from([
                    (
                        1,
                        TransactionLog {
                            amount: TransactionActionState::Deposit { amount: dec!(1000) },
                            state: TransactionState::Resolve,
                        },
                    ),
                    (
                        2,
                        TransactionLog {
                            amount: TransactionActionState::Deposit { amount: dec!(1000) },
                            state: TransactionState::Dispute,
                        },
                    )
                ]),
            })
        );
    }

    #[test]
    fn deposit_data_should_be_disputed_if_that_data_is_resolved() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Dispute);

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(0),
                held: dec!(100),
                locked: false,
                transaction_log: HashMap::from([(
                    1,
                    TransactionLog {
                        amount: TransactionActionState::Deposit { amount: dec!(100) },
                        state: TransactionState::Dispute,
                    },
                )]),
            })
        );
    }

    #[test]
    fn disputed_deposit_data_should_be_resolved_if_resovle_transaction_data_come() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Dispute);
        accounts.add_transaction(1, 1, Transaction::Resolve);

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(100),
                held: dec!(0),
                locked: false,
                transaction_log: HashMap::from([(
                    1,
                    TransactionLog {
                        amount: TransactionActionState::Deposit { amount: dec!(100) },
                        state: TransactionState::Resolve,
                    },
                )]),
            })
        );
    }

    #[test]
    fn deposite_data_should_be_charge_back_if_that_data_is_disputed() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Dispute);
        accounts.add_transaction(1, 1, Transaction::Chargeback);

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(0),
                held: dec!(0),
                locked: true,
                transaction_log: HashMap::from([(
                    1,
                    TransactionLog {
                        amount: TransactionActionState::Deposit { amount: dec!(100) },
                        state: TransactionState::Chargeback,
                    },
                )]),
            })
        );
    }

    #[test]
    fn withdrawal_data_should_be_charge_back_if_that_data_is_disputed() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 2, Transaction::Withdrawal { amount: dec!(100) });
        accounts.add_transaction(1, 2, Transaction::Dispute);
        accounts.add_transaction(1, 2, Transaction::Chargeback);

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(0),
                held: dec!(0),
                locked: true,
                transaction_log: HashMap::from([
                    (
                        1,
                        TransactionLog {
                            amount: TransactionActionState::Deposit { amount: dec!(100) },
                            state: TransactionState::Resolve,
                        },
                    ),
                    (
                        2,
                        TransactionLog {
                            amount: TransactionActionState::Withdrawal { amount: dec!(100) },
                            state: TransactionState::Chargeback,
                        },
                    ),
                ]),
            })
        );
    }

    #[test]
    fn account_should_be_frozen_if_account_is_locked() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 1, Transaction::Dispute);
        accounts.add_transaction(1, 1, Transaction::Chargeback);
        //after chargeback, account is fronze. that means transactions after chargeback should be ignored
        accounts.add_transaction(1, 2, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 3, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(1, 4, Transaction::Deposit { amount: dec!(100) });

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(0),
                held: dec!(0),
                locked: true,
                transaction_log: HashMap::from([(
                    1,
                    TransactionLog {
                        amount: TransactionActionState::Deposit { amount: dec!(100) },
                        state: TransactionState::Chargeback,
                    },
                )]),
            })
        );
    }
}
