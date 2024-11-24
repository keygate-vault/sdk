# Keygate SDK

A Rust SDK for interacting with Internet Computer Protocol (ICP) wallets through the Keygate canister. This SDK provides a simple interface for creating and managing ICP wallets, checking balances, and executing transactions.

## Features

- Create new ICP wallets (with optional CSV logging)
- Get wallet account IDs
- Check ICP balances
- Execute ICP transactions
- Python bindings through PyO3

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
keygate-sdk = "0.1.0"
```

## Usage

### Creating a Client

```rust
use keygate_sdk::{KeygateClient, load_identity};

#[tokio::main]
async fn main() {
    // Load identity from PEM file
    let identity = load_identity("path/to/identity.pem").await.unwrap();
    
    // Create client
    let client = KeygateClient::new(identity, "https://your-keygate-url").await.unwrap();
}
```

### Creating a Wallet

```rust
// Create wallet and save ID to wallets.csv
let wallet_id = client.create_wallet_write_file().await.unwrap();

// Create wallet without saving to file
let wallet_id = client.create_wallet().await.unwrap();
```

### Getting Account Information

```rust
// Get ICP account ID
let account_id = client.get_icp_account(&wallet_id.to_string()).await.unwrap();

// Get ICP balance
let balance = client.get_icp_balance(&wallet_id.to_string()).await.unwrap();
```

### Executing Transactions

```rust
use keygate_sdk::TransactionArgs;

let transaction = TransactionArgs {
    to: "recipient_account_id".to_string(),
    amount: 1.5, // Amount in ICP
};

let status = client
    .execute_transaction(&wallet_id.to_string(), &transaction)
    .await
    .unwrap();
```

## Transaction Status

Transactions can have the following statuses:

- `Pending`: Transaction is waiting to be processed
- `InProgress`: Transaction is being processed
- `Completed`: Transaction has been successfully completed
- `Rejected`: Transaction was rejected
- `Failed`: Transaction failed to process

## Error Handling

The SDK uses Rust's standard `Error` type for error handling. All main functions return a `Result<T, Error>` which should be properly handled in your application.

## Python Bindings

This SDK includes Python bindings through PyO3, allowing you to use the SDK in Python applications. Documentation for Python usage will be provided separately.

## Security Considerations

- Keep your PEM file secure and never share it
- Store wallet IDs safely - they are required for all wallet operations
- Always verify transaction details before execution

## Development

### Prerequisites

- Rust 1.54 or higher
- Cargo
- An Internet Computer identity (PEM file)
- Access to a Keygate canister

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## License
MIT License
