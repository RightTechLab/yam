use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct YamConfig {
    pub rpc_host: String,
    pub rpc_user: String,
    pub rpc_pass: String,
    pub bitcoin_conf_path: String,
    pub tor_bitcoin_hostname_path: String,
    pub tor_electrs_hostname_path: String,
    pub tor_mempool_hostname_path: String,
    pub tor_explorer_hostname_path: String,
}

impl Default for YamConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let bitcoin_dir = home.join(".bitcoin");

        Self {
            rpc_host: "http://127.0.0.1:8332".into(),
            rpc_user: "bitcoin".into(),
            rpc_pass: "bitcoin".into(),
            bitcoin_conf_path: bitcoin_dir.join("bitcoin.conf").to_string_lossy().into_owned(),
            tor_bitcoin_hostname_path: default_tor_hostname_path("bitcoinrpc"),
            tor_electrs_hostname_path: default_tor_hostname_path("electrs"),
            tor_mempool_hostname_path: default_tor_hostname_path("mempool"),
            tor_explorer_hostname_path: default_tor_hostname_path("bitcoinexplorer"),
        }
    }
}

fn default_tor_hostname_path(service_dir: &str) -> String {
    #[cfg(target_os = "macos")]
    let path = PathBuf::from("/usr/local/var/lib/tor")
        .join(service_dir)
        .join("hostname");

    #[cfg(not(target_os = "macos"))]
    let path = PathBuf::from("/var/lib/tor")
        .join(service_dir)
        .join("hostname");

    path.to_string_lossy().into_owned()
}

impl YamConfig {
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".yam").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
