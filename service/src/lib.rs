pub mod service {
    use domain::domain::{Accounts, Transaction};
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use std::{error::Error, fs::File};

    const DEPOSIT: &str = "deposit";
    const WITHDRAWAL: &str = "withdrawal";
    const DISPUTE: &str = "dispute";
    const RESOLVE: &str = "resolve";
    const CHARGEBACK: &str = "chargeback";

    #[derive(Debug, Deserialize)]
    struct InputTransactionRecord {
        #[serde(rename = "type")]
        transaction_type: String,
        #[serde(rename = "client")]
        client: u16,
        tx: u32,
        amount: Option<Decimal>,
    }
    impl InputTransactionRecord {
        fn convert(&self) -> Option<Transaction> {
            match self.transaction_type.as_str() {
                DEPOSIT => self.amount.map(|x| Transaction::Deposit { amount: x }),
                WITHDRAWAL => self.amount.map(|x| Transaction::Withdrawal { amount: x }),
                DISPUTE => Option::Some(Transaction::Dispute),
                RESOLVE => Option::Some(Transaction::Resolve),
                CHARGEBACK => Option::Some(Transaction::Chargeback),
                _ => Option::None,
            }
        }
    }

    #[derive(Debug, Serialize)]
    struct OutputRecord {
        client: u16,
        available: Decimal,
        held: Decimal,
        total: Decimal,
        locked: bool,
    }

    pub fn read_csv(file_path: String) -> Result<Accounts, Box<dyn Error>> {
        let mut rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(File::open(file_path)?);

        let mut accounts = Accounts::new();

        for result in rdr.deserialize() {
            let record: InputTransactionRecord = result?;
            if let Some(transaction) = record.convert() {
                accounts.add_transaction(record.client, record.tx, transaction);
            }
        }

        Ok(accounts)
    }

    pub fn write_csv(file_path: String, accounts: &Accounts) -> Result<(), Box<dyn Error>> {
        println!("client,available,held,total,lock");
        let mut wtr = csv::Writer::from_path(file_path)?;

        accounts.get_user_accounts().for_each(|item| {
            let record = OutputRecord {
                client: item.0.clone(),
                available: item.1.available.round_dp(4),
                held: item.1.held.round_dp(4),
                total: item.1.available.round_dp(4) + item.1.held.round_dp(4),
                locked: item.1.locked,
            };
            println!(
                "{},{},{},{},{}",
                record.client, record.available, record.held, record.total, record.locked
            );
            wtr.serialize(record).expect("fail to serialize");
            wtr.flush().expect("fail to serialize");
        });

        Ok(())
    }
}
