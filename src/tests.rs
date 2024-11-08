#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn test_get_icp_balance() -> Result<()> {
        let identity = load_identity("identity.pem").await?;
        let keygate = KeygateClient::new(identity, "https://ic0.app").await?;
        let wallet_id = keygate.create_wallet().await?;
        let balance = keygate.get_icp_balance(&wallet_id.to_string()).await?;
        assert!(balance > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_wallet() -> Result<()> {
        let identity = load_identity("identity.pem").await?;
        let keygate = KeygateClient::new(identity, "https://ic0.app").await?;
        let wallet_id = keygate.create_wallet().await?;
        assert!(!wallet_id.to_string().is_empty());
        Ok(())
    }
}
