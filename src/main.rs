use ic_agent::{Agent, Identity};
use anyhow::Result;

pub async fn create_agent(url: &str, is_mainnet: bool) -> Result<Agent> {
    let agent = Agent::builder().with_url(url).build()?;
    if !is_mainnet {
        agent.fetch_root_key().await?;
    }
    Ok(agent)
}

pub async fn load_identity(path: &str) -> Result<Identity> {
    // We're going to ignore the PEM file for now
    let identity = Identity::new
    Ok(identity)
}

fn main() {
    let 
    println!("Hello, world!");
}
