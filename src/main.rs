use std::io::Error;

use candid::CandidType;
use ic_agent::export::PrincipalError;
use ic_agent::{export::Principal, identity::Secp256k1Identity, Agent, Identity};
use ic_utils::call::{AsyncCall, SyncCall};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Canister;
use serde::Deserialize;

#[cfg(test)]
mod tests;

pub async fn create_agent(url: &str, is_mainnet: bool) -> Result<Agent, Error> {
    let agent = Agent::builder().with_url(url).build().unwrap();
    if !is_mainnet {
        agent.fetch_root_key().await.unwrap();
    }
    Ok(agent)
}

pub async fn load_identity(path: &str) -> Result<Secp256k1Identity, Error> {
    let identity = Secp256k1Identity::from_pem_file(path);
    match identity {
        Ok(identity) => Ok(identity),
        Err(e) => Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
    }
}

pub struct KeygateClient {
    agent: Agent,
}

fn gzip(blob: Vec<u8>) -> Result<Vec<u8>, Error> {
    use libflate::gzip::Encoder;
    use std::io::Write;
    let mut encoder = Encoder::new(Vec::with_capacity(blob.len())).unwrap();
    encoder.write_all(&blob).unwrap();
    Ok(encoder.finish().into_result().unwrap())
}

#[derive(CandidType, Deserialize)]
struct ICPAccountBalanceArgs {
    account: Vec<u8>,
}

impl KeygateClient {
    pub async fn new(identity: Secp256k1Identity, url: &str) -> Result<Self, Error> {
        let agent = Agent::builder()
            .with_url(url)
            .with_identity(identity)
            .build()
            .unwrap();
        agent.fetch_root_key().await.unwrap();
        Ok(Self { agent })
    }

    pub async fn get_icp_balance(&self, wallet_id: &str) -> Result<Vec<u8>, Error> {
        // Define the canister ID for the ledger
        let canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
        
        // Replace "your_account_identifier" with the actual account identifier
        let account_identifier = self.get_icp_account(wallet_id).await?;

        let args = ICPAccountBalanceArgs {
            account: account_identifier.as_bytes().to_vec(),
        };

        // Encode the account identifier in the format required by the ledger
        let encoded_args = candid::encode_args((args,)).unwrap();

        // Perform the query
        let query = self
            .agent
            .query(&canister_id, "account_balance")
            .with_arg(encoded_args) // Pass the encoded account identifier
            .call()
            .await.unwrap();

        // self.agent.get_principal().await?;
        Ok(query)
    }

    pub async fn create_wallet(&self) -> Result<Principal, Error> {
        // deploy a new wallet
        let management_canister = ManagementCanister::from_canister(
            Canister::builder()
                .with_agent(&self.agent)
                .with_canister_id("aaaaa-aa")
                .build()
                .unwrap(),
        );

        let (new_canister_id,) = management_canister
            .create_canister()
            .as_provisional_create_with_amount(None)
            .with_effective_canister_id(
                Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap(),
            )
            .call_and_wait()
            .await
            .unwrap();

        let (status,) = management_canister
            .canister_status(&new_canister_id)
            .call_and_wait()
            .await
            .unwrap();

        assert_eq!(format!("{}", status.status), "Running");

        let account_wasm = gzip(include_bytes!("./account.wasm").to_vec()).unwrap();

        management_canister
            .install_code(&new_canister_id, &account_wasm)
            .call_and_wait()
            .await
            .unwrap();

        let canister = Canister::builder()
            .with_agent(&self.agent)
            .with_canister_id(new_canister_id)
            .build()
            .unwrap();

        return Ok(*canister.canister_id());
    }

    pub async fn get_icp_account(&self, wallet_id: &str) -> Result<String, Error> {
        let wallet = Canister::builder()
            .with_agent(&self.agent)
            .with_canister_id(wallet_id)
            .build()
            .unwrap();
        let account_id: (String, ) = wallet.query("get_icp_account").build().call().await.unwrap();
        Ok(account_id.0)
    }

    pub async fn execute_transaction(&self, wallet_id: &str, transaction: &str) -> Result<String, Error> {
        panic!("Not implemented");
    }
}

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

    Ok(())
}
