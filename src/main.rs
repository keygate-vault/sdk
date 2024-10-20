use ic_agent::{identity::{Secp256k1Identity}, Agent};
use anyhow::Result;

pub async fn create_agent(url: &str, is_mainnet: bool) -> Result<Agent> {
    let agent = Agent::builder().with_url(url).build()?;
    if !is_mainnet {
        agent.fetch_root_key().await?;
    }
    Ok(agent)
}

pub async fn load_identity(path: &str) -> Result<Secp256k1Identity> {
    // We're going to ignore the PEM file for now
    let identity = Secp256k1Identity::from_pem_file(path);
    match identity {
        Ok(identity) => Ok(identity),
        Err(e) => anyhow::bail!("Failed to load identity: {}", e),
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let identity = load_identity("identity.pem").await?;
    println!("Hello, world!");
    Ok(())
}
