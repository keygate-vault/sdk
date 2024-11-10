use candid::Principal;
use keygate_sdk::{
    load_identity, KeygateClient, ProposeTransactionArgs, SupportedNetwork, TransactionType,
};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!(
        "\nWelcome to the Keygate Client CLI! \n\nPlease wait while we create your wallet...\n"
    );
    sleep(Duration::from_secs(2)).await;

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

    let address = keygate
        .get_icp_account(&wallet_id.to_string())
        .await
        .unwrap();

    let output = Command::new("dfx")
        .args(&[
            "ledger",
            "transfer",
            &address.to_string(),
            "--amount",
            "100",
            "--memo",
            "1",
            "--network",
            "local",
            "--identity",
            "minter",
            "--fee",
            "0",
        ])
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Wallet funded with 100 ICP!");
    } else {
        eprintln!("Failed to fund wallet: {:?}", output);
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
                check_balance(&keygate, wallet_id.clone()).await;
            }
            "transact" => {
                if input.len() < 4 {
                    eprintln!("Usage: transact <address> <amount>");
                    continue;
                }
                send_transaction(&keygate, wallet_id.clone(), input[1], input[2]).await;
            }
            "address" => {
                println!("Your account ID is: {}", address);
            }
            "exit" => {
                println!("Exiting...");
                break;
            }
            _ => eprintln!("Unknown wallet command: {}", input.join(" ")),
        }
    }
}

async fn check_balance(keygate: &KeygateClient, wallet_id: Principal) {
    print!("Checking balance... ");
    let balance = keygate
        .get_icp_balance(&wallet_id.to_string())
        .await
        .unwrap();
    println!("Your balance is: {:?}", balance);
}

async fn send_transaction(
    keygate: &KeygateClient,
    wallet_id: Principal,
    address: &str,
    amount: &str,
) {
    print!("Setting up transaction... ");
    let transaction = ProposeTransactionArgs {
        to: address.to_string(),
        token: "icp:native".to_string(),
        transaction_type: TransactionType::Transfer,
        network: SupportedNetwork::ICP,
        amount: amount.parse().unwrap(),
    };
    println!("Sending transaction... ");
    let executed_transaction = keygate
        .execute_transaction(&wallet_id.to_string(), &transaction)
        .await;
    println!("Transaction sent: {:?}", executed_transaction);
}
