use chrono::Utc;
use clap::{arg, command, value_parser, Command};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Account {
    name: String,
    balance: f64,
    transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    date: String,
    description: String,
    amount: f64,
    transaction_type: TransactionType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
}

impl Account {
    fn new(name: String, balance: f64) -> Self {
        Account {
            name,
            balance,
            transactions: Vec::new(),
        }
    }

    fn deposit(&mut self, transaction: Transaction) {
        self.balance += transaction.amount;
        self.transactions.push(transaction);
    }

    fn withdraw(&mut self, transaction: Transaction) {
        if self.balance >= transaction.amount {
            self.balance -= transaction.amount;
            self.transactions.push(transaction);
        } else {
            println!("Insufficient funds.")
        }
    }

    fn transfer_to(&mut self, other_account: &mut Account, amount: f64) {
        if self.balance > -amount {
            self.balance -= amount;
            other_account.balance += amount;
            self.transactions.push(Transaction {
                amount,
                description: format!("Transfer to {}", other_account.name),
                date: Utc::now().format("%Y-%m-%d").to_string(),
                transaction_type: TransactionType::Transfer,
            });
            other_account.transactions.push(Transaction {
                amount,
                description: format!("Transfer from {}", self.name),
                date: Utc::now().format("%Y-%m-%d").to_string(),
                transaction_type: TransactionType::Transfer,
            })
        } else {
            println!("Insufficient funds.");
        }
    }

    fn display_history(&self) {
        println!("Transaction history for {}", self.name);
        for transaction in &self.transactions {
            println!(
                "{} - ${:.2} - {}",
                transaction.date, transaction.amount, transaction.description,
            );
        }
    }
}

fn add_account(accounts: &mut Vec<Account>, name: String, balance: f64, description: String) {
    let mut account = Account::new(name, balance);
    println!("new account {}", account.name);
    let transaction = Transaction {
        amount: balance,
        description,
        date: Utc::now().format("%Y-%m-%d").to_string(),
        transaction_type: TransactionType::Deposit,
    };
    account.transactions.push(transaction);
    accounts.push(account);
}

fn deposit_funds(accounts: &mut Vec<Account>, name: String, amount: f64, description: String) {
    if let Some(account) = accounts.iter_mut().find(|a| a.name == name) {
        let transaction = Transaction {
            amount,
            description,
            date: Utc::now().format("%Y-%m-%d").to_string(),
            transaction_type: TransactionType::Deposit,
        };
        account.deposit(transaction);
        println!("successful");
    } else {
        println!("account not found");
    }
}

fn withdraw_funds(accounts: &mut Vec<Account>, name: String, amount: f64, description: String) {
    if let Some(account) = accounts.iter_mut().find(|a| a.name == name) {
        let transaction = Transaction {
            amount,
            description,
            date: Utc::now().format("%Y-%m-%d").to_string(),
            transaction_type: TransactionType::Withdrawal,
        };
        account.withdraw(transaction);
        println!("successful");
    } else {
        println!("account not found");
    }
}

fn transfer_funds(
    accounts: &mut Vec<Account>,
    source_name: String,
    destination_name: String,
    amount: f64,
) {
    let mut source_account = None;
    let mut destination_account = None;

    for account in accounts.iter_mut() {
        if account.name == source_name {
            source_account = Some(account);
        } else if account.name == destination_name {
            destination_account = Some(account);
        }

        if source_account.is_some() && destination_account.is_some() {
            break;
        }
    }

    if let (Some(source), Some(destination)) = (source_account, destination_account) {
        source.transfer_to(destination, amount);
        println!("successful");
    } else {
        println!("account(s) not found");
    }
}

fn display_transaction_history(accounts: &Vec<Account>, name: String) {
    if let Some(account) = accounts.iter().find(|a| a.name == name) {
        account.display_history();
    } else {
        println!("`{}` not found.", name);
    }
}

fn save_accounts(accounts: &Vec<Account>) {
    let mut bin_dir = PathBuf::from(env::var("HOME").unwrap());
    bin_dir.push("bin");
    bin_dir.push("accounts.json");

    let serialized = serde_json::to_string(&accounts).expect("Failed to serialize accounts.");
    let mut file = File::create(bin_dir).expect("Failed to create file.");
    file.write_all(serialized.as_bytes())
        .expect("Failed to write to file.");
}

fn load_accounts() -> Vec<Account> {
    let mut bin_dir = PathBuf::from(env::var("HOME").unwrap());
    bin_dir.push("bin");
    bin_dir.push("accounts.json");

    let file = File::open(bin_dir);
    match file {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Failed to read file.");
            serde_json::from_str(&contents).expect("Failed to deserialize accounts.")
        }
        Err(_) => Vec::new(),
    }
}

fn display_accounts(accounts: &Vec<Account>) {
    if accounts.is_empty() {
        println!("No accounts found.");
    } else {
        let mut sorted_accounts = accounts.clone();
        sorted_accounts.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap());

        for account in sorted_accounts {
            println!("${:.2} {}", account.balance, account.name);
        }

        println!(
            "${:.2} Total",
            accounts
                .iter()
                .fold(0.0, |acc, account| acc + account.balance)
        )
    }
}

fn main() {
    let matches = command!()
        .name("Finn - Personal Finance")
        .version("1.0")
        .author("Luisbert Seijas")
        .about("CLI personal finance management tool")
        .subcommand(
            Command::new("add")
                .about("add a new account")
                .arg(arg!(<NAME> "account name").required(true))
                .arg(
                    arg!(<BALANCE> "initial balance")
                        .required(true)
                        .value_parser(value_parser!(f64)),
                )
                .arg(arg!(<DESCRIPTION> "description").required(true)),
        )
        .subcommand(
            Command::new("deposit")
                .about("deposit funds into account")
                .arg(arg!(<NAME> "account name").required(true))
                .arg(
                    arg!(<AMOUNT> "deposit ammount")
                        .required(true)
                        .value_parser(value_parser!(f64)),
                )
                .arg(arg!(<DESCRIPTION> "transaction description").required(true)),
        )
        .subcommand(
            Command::new("withdraw")
                .about("Withdraw funds from an account")
                .arg(arg!(<NAME> "Account name").required(true))
                .arg(
                    arg!(<AMOUNT> "Withdrawal amount")
                        .required(true)
                        .value_parser(value_parser!(f64)),
                )
                .arg(arg!(<DESCRIPTION> "Transaction description").required(true)),
        )
        .subcommand(
            Command::new("transfer")
                .about("Transfer funds between accounts")
                .arg(arg!(<SOURCE> "Source account name").required(true))
                .arg(arg!(<DESTINATION> "Destination account name").required(true))
                .arg(
                    arg!(<AMOUNT> "Transfer amount")
                        .required(true)
                        .value_parser(value_parser!(f64)),
                ),
        )
        .subcommand(
            Command::new("history")
                .about("Display transaction history for an account")
                .arg(arg!(<NAME> "Account name").required(true)),
        )
        .get_matches();

    let mut accounts: Vec<Account> = load_accounts();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap().clone();
            let balance = *sub_matches.get_one::<f64>("BALANCE").unwrap();
            let description = sub_matches
                .get_one::<String>("DESCRIPTION")
                .unwrap()
                .clone();
            add_account(&mut accounts, name, balance, description);
        }
        Some(("deposit", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap().clone();
            let amount = *sub_matches.get_one::<f64>("AMOUNT").unwrap();
            let description = sub_matches
                .get_one::<String>("DESCRIPTION")
                .unwrap()
                .clone();
            deposit_funds(&mut accounts, name, amount, description);
        }
        Some(("withdraw", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap().clone();
            let amount = *sub_matches.get_one::<f64>("AMOUNT").unwrap();
            let description = sub_matches
                .get_one::<String>("DESCRIPTION")
                .unwrap()
                .clone();
            withdraw_funds(&mut accounts, name, amount, description);
        }
        Some(("transfer", sub_matches)) => {
            let source_name = sub_matches.get_one::<String>("SOURCE").unwrap().clone();
            let dest_name = sub_matches
                .get_one::<String>("DESTINATION")
                .unwrap()
                .clone();
            let amount = *sub_matches.get_one::<f64>("AMOUNT").unwrap();
            transfer_funds(&mut accounts, source_name, dest_name, amount);
        }
        Some(("history", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap().clone();
            display_transaction_history(&accounts, name);
        }
        None => {
            display_accounts(&accounts);
        }
        _ => unreachable!(),
    }

    save_accounts(&accounts);
}
