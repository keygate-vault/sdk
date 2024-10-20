use ic_agent::{export::Principal, identity::Secp256k1Identity, Agent, Identity};
use anyhow::{Error, Result};

pub async fn create_agent(url: &str, is_mainnet: bool) -> Result<Agent> {
    let agent = Agent::builder().with_url(url).build()?;
    if !is_mainnet {
        agent.fetch_root_key().await?;
    }
    Ok(agent)
}

pub async fn load_identity(path: &str) -> Result<Secp256k1Identity> {
    let identity = Secp256k1Identity::from_pem_file(path);
    match identity {
        Ok(identity) => Ok(identity),
        Err(e) => anyhow::bail!("Failed to load identity: {}", e),
    }
}

pub struct KeygateClient {
    agent: Agent,
}

impl KeygateClient {
    pub async fn new(identity: Secp256k1Identity, url: &str) -> Result<Self, Error> {
        let agent = Agent::builder()
            .with_url(url)
            .with_identity(identity)
            .build()?;
        agent.fetch_root_key().await?;
        Ok(Self { agent })
    }

    pub async fn create_wallet(&self) -> Result<(Principal, String)> {
        panic!("Not implemented");
    }

    pub async fn get_icp_account(&self, wallet_id: &str) -> Result<String> {
        panic!("Not implemented");
    }

    pub async fn execute_transaction(&self, wallet_id: &str, transaction: &str) -> Result<String> {
        panic!("Not implemented");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let identity = load_identity("identity.pem").await?;
    let keygate = KeygateClient::new(identity, "https://ic0.app").await?;
    
    println!("Hello, world!");
    Ok(())
}
