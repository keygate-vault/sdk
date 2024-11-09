use std::cell::RefCell;
use std::io::Error;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use candid::{CandidType, Decode};
use ic_agent::agent::CallResponse;
use ic_agent::{export::Principal, identity::Secp256k1Identity, Agent};
use ic_ledger_types::{AccountBalanceArgs, AccountIdentifier, DEFAULT_SUBACCOUNT};
use ic_utils::call::{AsyncCall, SyncCall};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Canister;
use serde::{Deserialize, Serialize};

use pyo3::prelude::*;

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

#[pyclass]
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

#[derive(CandidType, Deserialize)]
struct BalanceResponse {
    e8s: u64,
}

#[derive(Deserialize, CandidType, Serialize, Debug, Clone, PartialEq)]
pub enum TransactionType {
    Swap,
    Transfer,
}
#[derive(Deserialize, Serialize, CandidType, Debug, Clone, PartialEq)]
pub enum SupportedNetwork {
    ICP,
    ETH,
}

pub type TokenPath = String;

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ProposeTransactionArgs {
    pub to: String,
    pub token: TokenPath,
    pub transaction_type: TransactionType,
    pub network: SupportedNetwork,
    pub amount: f64,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ProposedTransaction {
    pub id: u64,
    pub to: String,
    pub token: TokenPath,
    pub network: SupportedNetwork,
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub signers: Vec<Principal>,
    pub rejections: Vec<Principal>,
}

#[derive(
    CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, strum_macros::IntoStaticStr,
)]
pub enum IntentStatus {
    Pending(String),
    InProgress(String),
    Completed(String),
    Rejected(String),
    Failed(String),
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

    pub async fn get_icp_balance(&self, wallet_id: &str) -> Result<u64, Error> {
        // Define the canister ID for the ledger
        let canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
        // // Replace "your_account_identifier" with the actual account identifier
        let account_identifier = AccountIdentifier::new(
            &Principal::from_str(&wallet_id).unwrap(),
            &DEFAULT_SUBACCOUNT,
        );

        let args = AccountBalanceArgs {
            account: account_identifier,
        };

        // // Encode the account identifier in the format required by the ledger
        let encoded_args = candid::encode_args((args,)).unwrap();
        // Perform the query
        let query = self
            .agent
            .query(&canister_id, "account_balance")
            .with_arg(encoded_args) // Pass the encoded account identifier
            .call()
            .await
            .unwrap();

        // self.agent.get_principal().await?;
        // Decode the response into the BalanceResponse struct
        let balance_response: BalanceResponse = Decode!(&query, BalanceResponse).unwrap();

        // Extract the balance in e8s
        Ok(balance_response.e8s)
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

        let account_id: (String,) = wallet
            .query("get_icp_account")
            .build()
            .call()
            .await
            .unwrap();
        Ok(account_id.0)
    }

    pub async fn execute_transaction(
        &self,
        wallet_id: &str,
        transaction: &ProposeTransactionArgs,
    ) -> Result<IntentStatus, Error> {
        let wallet = Canister::builder()
            .with_agent(&self.agent)
            .with_canister_id(wallet_id)
            .build()
            .unwrap();

        // let encoded_args = candid::encode_args((transaction.clone(),)).unwrap();
        let proposed_transaction: CallResponse<(ProposedTransaction,)> = wallet
            .update("propose_transaction")
            .with_arg(transaction.clone())
            .build()
            .call()
            .await
            .unwrap();
        let response: ProposedTransaction;
        match proposed_transaction {
            CallResponse::Response((proposed_transaction,)) => response = proposed_transaction,
            CallResponse::Poll(_) => {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    "Transaction is still pending",
                ));
            }
        };

        let threshold: (u64,) = wallet.query("get_threshold").build().call().await.unwrap();
        let intent_response: CallResponse<(IntentStatus,)>;
        if threshold.0 <= 1 {
            intent_response = wallet
                .update("execute_transaction")
                .with_arg(response.id)
                .build()
                .call()
                .await
                .unwrap();
        } else {
            return Err(Error::new(std::io::ErrorKind::Other, "Transaction failed"));
        }

        match intent_response {
            CallResponse::Response((status,)) => Ok(status),
            CallResponse::Poll(_) => Err(Error::new(
                std::io::ErrorKind::Other,
                "Transaction is still pending",
            )),
        }
    }
}

#[pyclass]
struct PyKeygateClient {
    identity_path: String,
    url: String,
    keygate: Arc<Mutex<Option<KeygateClient>>>,
}

#[pymethods]
impl PyKeygateClient {
    #[new]
    fn new(identity_path: &str, url: &str) -> PyResult<Self> {
        Ok(Self {
            identity_path: identity_path.to_string(),
            url: url.to_string(),
            keygate: Arc::new(Mutex::new(None)),
        })
    }

    fn init<'py>(&'py mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let identity_path = self.identity_path.clone();
        let url = self.url.clone();
        
        // Thread-safe mutable state
        // Keygate is now a Mutex: meaning that it can be accessed by multiple threads
        // And it is Arc: meaning that it can be shared between threads
        // So we can mutate it from multiple threads by locking it
        // And we can reference it from multiple threads by cloning the Arc (reference counting)
        let keygate_clone = self.keygate.clone();
        
        // Future_into_py takes a future and a Python context
        // and runs the future in the context of the Python thread
        // and returns the result of the future to the Python thread.
        // 
        // async move { ... } is used to capture the variables by value
        // and move them into the async block. This is necessary because the
        // future needs to capture the variables from the surrounding scope
        // and keep them alive for the duration of the future.
        //
        
        pyo3_asyncio::async_std::future_into_py(py, async move { 
            let identity = load_identity(&identity_path).await?;
            let client = KeygateClient::new(identity, &url).await?;
            
            // Update through Mutex instead of RefCell
            *keygate_clone.lock().unwrap() = Some(client);
            Ok(())
        })
    }
}

fn init_test<'py>(py: Python<'py>) -> PyResult<&PyAny> {
    pyo3_asyncio::async_std::future_into_py(py, async { Ok(()) })
}

#[pymodule]
fn keygate_sdk(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<KeygateClient>()?;
    Ok(())
}