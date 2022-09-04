mod domain {
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
    pub enum DepositState {
        Resolve,
        Dispute,
        Chargeback,
    }

    #[derive(Debug, PartialEq)]
    pub struct Deposit {
        pub amount: Decimal,
        pub state: DepositState,
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
        pub deposit_log: HashMap<u32, Deposit>,
    }

    impl UserAccount {
        fn new(tx: u32, transaction: Transaction) -> Option<UserAccount> {
            match transaction {
                Transaction::Deposit { amount } => Option::Some(UserAccount {
                    available: amount,
                    held: dec!(0),
                    locked: false,
                    deposit_log: HashMap::from([(
                        tx,
                        Deposit {
                            amount: amount,
                            state: DepositState::Resolve,
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
                    self.deposit_log.insert(
                        tx,
                        Deposit {
                            amount: amount,
                            state: DepositState::Resolve,
                        },
                    );
                    self.available = self.available + amount;
                }

                Transaction::Dispute => {
                    if let Some(x) = self.deposit_log.get_mut(&tx) {
                        if matches!(x.state, DepositState::Resolve) {
                            *x = Deposit {
                                amount: x.amount,
                                state: DepositState::Dispute,
                            };
                            self.available = self.available - x.amount;
                            self.held = self.held + x.amount;
                        }
                    }
                }

                Transaction::Resolve => {
                    if let Some(x) = self.deposit_log.get_mut(&tx) {
                        if matches!(x.state, DepositState::Dispute) {
                            *x = Deposit {
                                amount: x.amount,
                                state: DepositState::Resolve,
                            };
                            self.available = self.available + x.amount;
                            self.held = self.held - x.amount;
                        }
                    }
                }

                Transaction::Chargeback => {
                    if let Some(x) = self.deposit_log.get_mut(&tx) {
                        if matches!(x.state, DepositState::Dispute) {
                            *x = Deposit {
                                amount: x.amount,
                                state: DepositState::Chargeback,
                            };
                            self.held = self.held - x.amount;
                            self.locked = true;
                        }
                    }
                }

                Transaction::Withdrawal { amount } => {
                    self.withdrawal(amount);
                }
            }
        }

        fn withdrawal(&mut self, amount: Decimal) {
            if self.available >= amount {
                self.available = self.available - amount;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rust_decimal_macros::dec;

    use crate::domain::{Accounts, Deposit, DepositState, Transaction, UserAccount};

    #[test]
    fn first_transaction_should_be_added_if_transaction_state_is_deposit() {
        let mut accounts = Accounts::new();
        accounts.add_transaction(1, 1, Transaction::Deposit { amount: dec!(100) });
        accounts.add_transaction(2, 2, Transaction::Deposit { amount: dec!(1000) });

        assert_eq!(
            accounts.get_user_account(1),
            Some(&UserAccount {
                available: dec!(100),
                held: dec!(0),
                locked: false,
                deposit_log: HashMap::from([(
                    1,
                    Deposit {
                        amount: dec!(100),
                        state: DepositState::Resolve,
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
                deposit_log: HashMap::from([(
                    2,
                    Deposit {
                        amount: dec!(1000),
                        state: DepositState::Resolve,
                    },
                )]),
            })
        );
    }

    #[test]
    fn diposit_and_withdrawal_should_be_ignored_if_same_transaction_id_is_already_existed() {
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
                deposit_log: HashMap::from([(
                    1,
                    Deposit {
                        amount: dec!(100),
                        state: DepositState::Resolve,
                    },
                )]),
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
                deposit_log: HashMap::from([
                    (
                        1,
                        Deposit {
                            amount: dec!(1000),
                            state: DepositState::Resolve,
                        },
                    ),
                    (
                        2,
                        Deposit {
                            amount: dec!(1000),
                            state: DepositState::Resolve,
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
                deposit_log: HashMap::from([
                    (
                        1,
                        Deposit {
                            amount: dec!(1000),
                            state: DepositState::Resolve,
                        },
                    ),
                    (
                        2,
                        Deposit {
                            amount: dec!(1000),
                            state: DepositState::Dispute,
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
                deposit_log: HashMap::from([(
                    1,
                    Deposit {
                        amount: dec!(100),
                        state: DepositState::Dispute,
                    },
                )]),
            })
        );
    }

    #[test]
    fn disputed_deposit_data_should_be_resolved_if_resovle_transaction_data_come(){
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
                deposit_log: HashMap::from([(
                    1,
                    Deposit {
                        amount: dec!(100),
                        state: DepositState::Resolve,
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
                deposit_log: HashMap::from([(
                    1,
                    Deposit {
                        amount: dec!(100),
                        state: DepositState::Chargeback,
                    },
                )]),
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
                deposit_log: HashMap::from([(
                    1,
                    Deposit {
                        amount: dec!(100),
                        state: DepositState::Chargeback,
                    },
                )]),
            })
        );
    }
}
