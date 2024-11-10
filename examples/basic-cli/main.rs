use candid::Principal;
use keygate_sdk::{load_identity, KeygateClient};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("Welcome to the Keygate Client CLI! \nPlease wait while we create your wallet...\n");
    sleep(Duration::from_secs(1)).await;

    println!("Loading identity...");
    let identity = load_identity("identity.pem").await.unwrap();
    println!("Identity loaded!");

    println!("Creating Keygate Client...");
    let keygate = KeygateClient::new(identity, "http://localhost:4943")
        .await
        .unwrap();
    println!("Keygate Client created!");

    println!("Creating wallet...");
    let wallet_id = keygate.create_wallet().await.unwrap();
    println!("Wallet created with ID: {}", wallet_id);

    let output = Command::new("dfx").args(&"ledger transfer <vault_account_id> --amount 100  --memo 1 --network local --identity minter --fee 0".split(' ').collect::<Vec<&str>>())
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Wallet funded with 100 ICP!");
    } else {
        eprintln!("Failed to fund wallet with 100 ICP!");
        return;
    }

    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input: Vec<&str> = input.trim().split(' ').collect();

        match input[1] {
            "balance" => {
                check_balance(keygate.clone(), wallet_id.clone()).await;
            }
            "transact" => {
                send_transaction(input[2], input[3]);
            }
            "address" => {
                println!("Your wallet address is: {}", wallet_id.to_string());
            }
            _ => eprintln!("Unknown wallet command: {}", input[2]),
        }
    }
}

async fn check_balance(keygate: KeygateClient, wallet_id: Principal) {
    print!("Checking balance... ");
    let balance = keygate
        .get_icp_balance(&wallet_id.to_string())
        .await
        .unwrap();
    println!("Your balance is: {:?}", balance);
}

fn send_transaction(keygate: KeygateClient, wallet_id: Principal, address: &str, amount: &str) {
    let transaction = ProposeTransactionArgs {
        to: account_id.to_string(),
        token: "icp:native".to_string(),
        transaction_type: TransactionType::Transfer,
        network: SupportedNetwork::ICP,
        amount: f64::from(0),
    };
    let executed_transaction = keygate
        .execute_transaction(&wallet_id.to_string(), &transaction)
        .await?;
    println!("Transaction status: {:?}", executed_transaction);
}
