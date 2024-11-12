use candid::Principal;
use keygate_sdk::{load_identity, IntentStatus, KeygateClient, TransactionArgs};
use std::io::Write;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

fn get_string_from_intent_status(status: IntentStatus) -> String {
    match status {
        IntentStatus::Pending(s)
        | IntentStatus::InProgress(s)
        | IntentStatus::Completed(s)
        | IntentStatus::Rejected(s)
        | IntentStatus::Failed(s) => s,
    }
}

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

    println!("\n\nKeygate CLI is ready! Type 'help' for a list of commands.\n");

    loop {
        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin()
            .read_line(&mut input)
            .expect("\nFailed to read line\n");
        let input: Vec<&str> = input.trim().split(' ').collect();

        match input[0] {
            "balance" => {
                check_balance(&keygate, wallet_id.clone()).await;
            }
            "transact" => {
                if input.len() < 3 {
                    eprintln!("\nUsage: transact <address> <amount>\n");
                    continue;
                }
                send_transaction(&keygate, wallet_id.clone(), input[1], input[2]).await;
            }
            "address" => {
                println!("\nYour account ID is: {}\n", address);
            }
            "help" => {
                println!("\nAvailable commands:");
                println!("balance - Check your balance");
                println!("transact <address> <amount> - Send ICP to an address");
                println!("address - Get your account ID");
                println!("exit - Exit the wallet\n");
            }
            "exit" => {
                println!("\nExiting...\n");
                break;
            }
            _ => eprintln!("\nUnknown wallet command: {}\n", input.join(" ")),
        }
    }
}

async fn check_balance(keygate: &KeygateClient, wallet_id: Principal) {
    print!("\nChecking balance... \n");
    let balance = keygate
        .get_icp_balance(&wallet_id.to_string())
        .await
        .unwrap();
    println!("Your balance is: {:?}\n", balance);
}

async fn send_transaction(
    keygate: &KeygateClient,
    wallet_id: Principal,
    address: &str,
    amount: &str,
) {
    print!("\nSetting up transaction... \n");
    let transaction = TransactionArgs {
        to: address.to_string(),
        amount: amount.parse().unwrap(),
    };
    println!("Sending transaction...\n");
    let executed_transaction = keygate
        .execute_transaction(&wallet_id.to_string(), &transaction)
        .await
        .unwrap();
    println!(
        "Transaction sent: {:?}\n",
        get_string_from_intent_status(executed_transaction)
    );
}
