# Keygate SDK

## Introduction

Keygate is an open-source service that provides a secure and decentralized way to manage your assets. Keygate SDK is a **Rust** and **Python** library that allows developers to interact with the Keygate service.

As of now, Keygate SDK supports the **ICP blockchain**. This SDK provides a simple interface to create wallets, get account information, and execute transactions.

## Installation

In order to use the Keygate SDK, you'll need to have `dfx` installed on your machine. `dfx` is natively supported on Linux and macOS (^12.x) and can be installed on Windows using WSL2 (Windows Subsystem for Linux). You can install `dfx` by following the instructions [here](https://internetcomputer.org/docs/current/developer-docs/getting-started/install/).

### Rust

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
keygate = "..."
```
 
### Python

Open a terminal and run the following command:

```bash
pip install keygate
```

## Example Usage

### Basic CLI

We've created a simple CLI in Rust to demonstrate how to use the Keygate SDK. In order to run the CLI, you need to have Rust installed on your machine. If you don't have Rust installed, you can install it by following the instructions [here](https://www.rust-lang.org/tools/install).

To run the CLI, you'll need to clone both this repository and the [multisignature](https://github.com/keygate-vault/multisignature) repository. Once you've cloned both repositories, you can run the CLI by following these steps:

1. Open a terminal on your machine and navigate to the directory where you cloned the [multisignature](https://github.com/keygate-vault/multisignature) repository.
2. You'll need to deploy the ledger canister on the IC network. You can do this by running the following command:
    
    ```bash
    ./ledger.sh
    ```
3. Once the ledger canister is deployed, you can open a terminal and head over to the directory where you cloned this repository.
4. Run the following command to build and run the CLI:

    ```bash
    cargo run --example basic-cli
    ```
5. That's it! You should now see the CLI running in your terminal. Follow the instructions in the CLI to create a wallet, get account information, and execute transactions.

