use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::json::{GetBlockchainInfoResult, GetMempoolInfoResult, GetNetworkInfoResult};
use std::path::PathBuf;

pub struct RpcClient {
    client: Client,
}

impl RpcClient {
    pub fn new() -> anyhow::Result<Self> {
        let _home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        
        #[cfg(target_os = "macos")]
        let cookie_path: PathBuf = ["/", "usr", "local", "var", "lib", "bitcoin", ".cookie"].iter().collect();
        
        #[cfg(target_os = "linux")]
        let cookie_path: PathBuf = [home.as_str(), ".bitcoin", ".cookie"].iter().collect();

        // Fallback or explicit override can be implemented here later
        let url = "http://127.0.0.1:18443"; // Default regtest RPC port

        let auth = Auth::CookieFile(cookie_path);

        match Client::new(url, auth) {
            Ok(client) => Ok(Self { client }),
            Err(e) => Err(anyhow::anyhow!("Failed to connect to Bitcoin RPC: {}", e)),
        }
    }

    pub fn with_auth(url: &str, auth: Auth) -> anyhow::Result<Self> {
        match Client::new(url, auth) {
            Ok(client) => Ok(Self { client }),
            Err(e) => Err(anyhow::anyhow!("Failed to connect to Bitcoin RPC: {}", e)),
        }
    }

    pub fn get_chain_info(&self) -> anyhow::Result<GetBlockchainInfoResult> {
        Ok(self.client.get_blockchain_info()?)
    }

    pub fn get_mempool_info(&self) -> anyhow::Result<GetMempoolInfoResult> {
        Ok(self.client.get_mempool_info()?)
    }

    pub fn get_network_info(&self) -> anyhow::Result<GetNetworkInfoResult> {
        Ok(self.client.get_network_info()?)
    }

    pub fn get_peer_count(&self) -> anyhow::Result<u64> {
        Ok(self.client.get_connection_count()? as u64)
    }
}
