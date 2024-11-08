use std::error::Error;

use keygate_sdk::load_identity;
use keygate_sdk::KeygateClient;
use keygate_sdk::ProposeTransactionArgs;
use keygate_sdk::TransactionType;
use keygate_sdk::SupportedNetwork;


#[tokio::main]
async fn main() -> Result<(), Error> {
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

    Ok(())
}
