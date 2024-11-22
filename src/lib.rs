use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Error;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use candid::{CandidType, Decode};
use ic_agent::agent::CallResponse;
use ic_agent::{export::Principal, identity::Secp256k1Identity, Agent};
use ic_ledger_types::{AccountBalanceArgs, AccountIdentifier, DEFAULT_SUBACCOUNT};
use ic_utils::call::{AsyncCall, SyncCall};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Canister;
use pyo3::types::PyString;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

use pyo3::prelude::*;

#[cfg(test)]
mod tests;

/// Load an identity from a PEM file.
/// ## Returns
/// A [`Result`] containing a [`Secp256k1Identity`] if the identity was loaded successfully, or an [`Error`] if an error occurred.
pub async fn load_identity(path: &str) -> Result<Secp256k1Identity, Error> {
    let identity = Secp256k1Identity::from_pem_file(path);
    match identity {
        Ok(identity) => Ok(identity),
        Err(e) => Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
    }
}

/// A client for interacting with the Keygate canister.
///
/// ## Features
/// - Create wallets
/// - Get account IDs
/// - Get balances
/// - Execute transactions
#[derive(Clone, Debug)]
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

#[derive(
    CandidType, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default,
)]
struct ICPAccountBalanceArgs {
    account: Vec<u8>,
}

#[derive(
    CandidType,
    Deserialize,
    Serialize,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
)]
struct BalanceResponse {
    e8s: u64,
}

#[derive(
    Deserialize, CandidType, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
enum TransactionType {
    // Swap,
    Transfer,
}
#[derive(
    Deserialize, Serialize, CandidType, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
enum SupportedNetwork {
    ICP,
    // ETH,
}

type TokenPath = String;

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
struct ProposeTransactionArgs {
    to: String,
    token: TokenPath,
    transaction_type: TransactionType,
    network: SupportedNetwork,
    amount: f64,
}

/// Transaction arguments for transferring in ICP
///
/// ## Fields
/// - `to` - The recipient's account ID
/// - `amount` - The amount to transfer
#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TransactionArgs {
    pub to: String,
    pub amount: f64,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
struct ProposedTransaction {
    id: u64,
    to: String,
    token: TokenPath,
    network: SupportedNetwork,
    amount: f64,
    transaction_type: TransactionType,
    signers: Vec<Principal>,
    rejections: Vec<Principal>,
}

/// Transaction status after execution
#[derive(
    CandidType,
    Deserialize,
    Serialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::IntoStaticStr,
)]
pub enum IntentStatus {
    Pending(String),
    InProgress(String),
    Completed(String),
    Rejected(String),
    Failed(String),
}

impl KeygateClient {
    /// Create a new Keygate client.
    /// ## Arguments
    /// * `identity` - A [`Secp256k1Identity`] that holds the user's identity. It can be created with the [`load_identity`] function.
    /// * `url` - A string slice that holds the URL of the Keygate canister.
    pub async fn new(identity: Secp256k1Identity, url: &str) -> Result<Self, Error> {
        let agent = Agent::builder()
            .with_url(url)
            .with_identity(identity)
            .build()
            .unwrap();
        agent.fetch_root_key().await.unwrap();
        Ok(Self { agent })
    }

    /// Get the ICP balance of an account.
    pub async fn get_icp_balance(&self, wallet_id: &str) -> Result<u64, Error> {
        let canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
        let account_identifier = AccountIdentifier::new(
            &Principal::from_str(&wallet_id).unwrap(),
            &DEFAULT_SUBACCOUNT,
        );

        let args = AccountBalanceArgs {
            account: account_identifier,
        };

        let encoded_args = candid::encode_args((args,)).unwrap();
        let query = self
            .agent
            .query(&canister_id, "account_balance")
            .with_arg(encoded_args)
            .call()
            .await
            .unwrap();

        let balance_response: BalanceResponse = Decode!(&query, BalanceResponse).unwrap();

        Ok(balance_response.e8s)
    }

    /// Used to create a new wallet. The wallet is a canister that holds the user's account.
    /// The wallet ID is written to a `wallets.csv` file. Running this function multiple times
    /// will create multiple wallets and its IDs will be appended to the file.
    /// ## Returns
    /// A [`Principal`] that holds the wallet ID.
    /// ## The wallet ID is needed to interact with the wallet. Don't lose it.
    pub async fn create_wallet_write_file(&self) -> Result<Principal, Error> {
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

        let canister_id_str = format!("{}", canister.canister_id());
        let csv_content = format!("{}\n", canister_id_str);

        let file_path = "./wallets.csv";
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();

        file.write_all(csv_content.as_bytes()).unwrap();

        return Ok(*canister.canister_id());
    }

    /// Used to create a new wallet. The wallet is a canister that holds the user's account.
    /// Running this function multiple times will create multiple wallets.
    /// ## Returns
    /// A [`Principal`] that holds the wallet ID.
    /// ## The wallet ID is needed to interact with the wallet. Don't lose it.
    pub async fn create_wallet(&self) -> Result<Principal, Error> {
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

    /// Get the account ID of your ICP account.
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

    /// Execute a transaction in ICP.
    /// ## Arguments
    /// * `wallet_id` - The ID of the wallet that holds the account.
    /// * `transaction` - A [`TransactionArgs`] that holds the transaction details.
    pub async fn execute_transaction(
        &self,
        wallet_id: &str,
        transaction: &TransactionArgs,
    ) -> Result<IntentStatus, Error> {
        let wallet = Canister::builder()
            .with_agent(&self.agent)
            .with_canister_id(wallet_id)
            .build()
            .unwrap();

        let complete_transaction = ProposeTransactionArgs {
            to: transaction.to.clone(),
            token: "icp:native".to_string(),
            transaction_type: TransactionType::Transfer,
            network: SupportedNetwork::ICP,
            amount: transaction.amount,
        };

        let proposed_transaction: CallResponse<(ProposedTransaction,)> = wallet
            .update("propose_transaction")
            .with_arg(complete_transaction.clone())
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

#[derive(Deserialize, Serialize)]
struct PersistedState {
    wallet_ids: HashSet<String>,
}

#[pyclass]
struct PyKeygateClient {
    identity_path: String,
    url: String,
    keygate: Arc<RwLock<Option<KeygateClient>>>,
    wallet_ids: Arc<RwLock<HashSet<String>>>,
}

impl PyKeygateClient {
    // Common initialization logic (not exposed to Python)
    async fn initialize(
        identity_path: &str,
        url: &str,
        keygate: Arc<RwLock<Option<KeygateClient>>>,
    ) -> PyResult<()> {
        let identity = load_identity(identity_path).await?;
        let client = KeygateClient::new(identity, url).await?;
        *keygate.write().unwrap() = Some(client);
        Ok(())
    }

    fn load_wallets(path: &PathBuf) -> HashSet<String> {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str::<PersistedState>(&content)
                .map(|sw| sw.wallet_ids)
                .unwrap(),
            Err(_) => HashSet::new(),
        }
    }

    fn save_wallets(&self) -> Result<(), std::io::Error> {
        let wallets = self.wallet_ids.read().unwrap();
        let state = PersistedState {
            wallet_ids: wallets.clone(),
        };
        let json = serde_json::to_string_pretty(&state)?;
        std::fs::write("config.json", json)?;
        Ok(())
    }
}

#[pymethods]
impl PyKeygateClient {
    #[new]
    fn new(identity_path: &str, url: &str) -> PyResult<Self> {
        Ok(Self {
            identity_path: identity_path.to_string(),
            url: url.to_string(),
            keygate: Arc::new(RwLock::new(None)),
            wallet_ids: Arc::new(RwLock::new(Self::load_wallets(&PathBuf::from(
                "config.json",
            )))),
        })
    }

    fn init<'py>(&'py mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let identity_path = self.identity_path.clone();
        let url = self.url.clone();
        let keygate = self.keygate.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            Self::initialize(&identity_path, &url, keygate).await
        })
    }

    fn create_wallet<'py>(&'py self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let keygate = self.keygate.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let client = {
                let guard = keygate.read().unwrap();
                guard.as_ref().cloned().expect("KeygateClient not initialized. Make sure to call init() before using other methods.")
            };

            let created_wallet_principal = client.create_wallet().await;

            match created_wallet_principal {
                Ok(principal) => {
                    let output = Command::new("dfx")
                        .args(&[
                            "ledger",
                            "transfer",
                            &client
                                .get_icp_account(&created_wallet_principal.unwrap().to_text())
                                .await
                                .unwrap(),
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
                    Ok(principal.to_text())
                }
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                    "Error creating a Keygate wallet: {}",
                    e
                ))),
            }
        })
    }

    #[pyo3(signature = (wallet_id))]
    fn get_icp_balance<'py>(&'py self, wallet_id: String, py: Python<'py>) -> PyResult<&'py PyAny> {
        if wallet_id.trim().is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Wallet ID cannot be empty",
            ));
        }

        let keygate = self.keygate.clone();

        println!("Getting ICP balance for wallet: {}", wallet_id);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let client = {
                let guard = keygate.read().unwrap();
                guard.as_ref().cloned().expect("KeygateClient not initialized. Make sure to call init() before using other methods.")
            };

            let balance = client.get_icp_balance(&wallet_id).await;

            match balance {
                Ok(balance) => Ok(balance),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                    "Error getting ICP balance: {}",
                    e
                ))),
            }
        })
    }

    fn get_icp_address<'py>(&'py self, wallet_id: &str, py: Python<'py>) -> PyResult<&'py PyAny> {
        let keygate = self.keygate.clone();
        let wallet_id = wallet_id.to_string();
        let client = {
            let guard = keygate.read().unwrap();
            guard.as_ref().cloned().expect("KeygateClient not initialized. Make sure to call init() before using other methods.")
        };

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let address = client.get_icp_account(&wallet_id).await;
            match address {
                Ok(address) => Ok(address),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                    "Error getting ICP address: {}",
                    e
                ))),
            }
        })
    }

    fn execute_transaction<'py>(
        &'py self,
        wallet_id: &str,
        to: &str,
        amount: f64,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let keygate = self.keygate.clone();
        let wallet_id = wallet_id.to_string();
        let transaction: TransactionArgs = TransactionArgs {
            to: to.to_string(),
            amount: amount,
        };
        let client = {
            let guard = keygate.read().unwrap();
            guard.as_ref().cloned().expect("KeygateClient not initialized. Make sure to call init() before using other methods.")
        };

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let status = client.execute_transaction(&wallet_id, &transaction).await;
            match status {
                Ok(status) => {
                    let status_str = match status {
                        IntentStatus::Pending(s) => s,
                        IntentStatus::InProgress(s) => s,
                        IntentStatus::Completed(s) => s,
                        IntentStatus::Rejected(s) => s,
                        IntentStatus::Failed(s) => s,
                    };
                    Ok(status_str)
                }
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                    "Error executing transaction: {}",
                    e
                ))),
            }
        })
    }
}

fn init_test<'py>(py: Python<'py>) -> PyResult<&PyAny> {
    pyo3_asyncio::async_std::future_into_py(py, async { Ok(()) })
}

#[pymodule]
fn keygate_sdk(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyKeygateClient>()?;
    Ok(())
}
