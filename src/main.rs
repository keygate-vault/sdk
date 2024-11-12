

use keygate_sdk::load_identity;
use keygate_sdk::KeygateClient;
use keygate_sdk::TransactionArgs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let identity = load_identity("identity.pem").await?;
    println!("Loaded identity.");

    //let keygate = KeygateClient::new(identity, "https://ic0.app").await?;
    let keygate = KeygateClient::new(identity, "http://localhost:4943").await?;
    println!("Created Keygate client.");

    let wallet_id = keygate.create_wallet().await?;
    println!("Created wallet with ID: {}", wallet_id);

    let account_id = keygate.get_icp_account(&wallet_id.to_string()).await?;
    println!("Account ID: {}", account_id);

    let balance = keygate.get_icp_balance(&wallet_id.to_string()).await?;
    println!("Balance: {:?}", balance);

    let transaction = TransactionArgs {
        to: account_id.to_string(),
        amount: f64::from(0),
    };
    let executed_transaction = keygate
        .execute_transaction(&wallet_id.to_string(), &transaction)
        .await?;
    println!("Transaction status: {:?}", executed_transaction);

    Ok(())
}
