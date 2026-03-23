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

    pub fn ensure_bitcoin_conf_exists(&self) -> anyhow::Result<()> {
        let path = std::path::PathBuf::from(&self.bitcoin_conf_path);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = "# Bitcoin Core\ndaemon=1\ntxindex=1\nblockfilterindex=1\ncoinstatsindex=1\n\n[main]\n# RPC\nserver=1\nrpcport=8332\nrpcbind=0.0.0.0\nrpcallowip=127.0.0.1\nrpcallowip=10.0.0.0/8\nrpcallowip=172.0.0.0/8\nrpcallowip=192.0.0.0/8\n\nzmqpubrawblock=tcp://0.0.0.0:28332\nzmqpubrawtx=tcp://0.0.0.0:28333\nzmqpubhashblock=tcp://0.0.0.0:28334\nwhitelist=127.0.0.1\n\n# Network\nlisten=1\nonlynet=onion\nonion=127.0.0.1:9050\nproxy=127.0.0.1:9050\nbind=127.0.0.1\n\ni2p=1\nonlynet=i2p\ni2pacceptincoming=1\ni2psam=127.0.0.1:7656\n\naddnode=etehks5xyh32nyjldpyeckk3nwpanivqhrzhsoracwqjxtk5apgq.b32.i2p:0\n[regtest]\nrpcport=18443\nrpcbind=127.0.0.1\nlisten=1\nserver=1\nonlynet=ipv4\nrpcallowip=127.0.0.1\n";
            std::fs::write(&path, content)?;
        }
        Ok(())
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let config: Self = match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        };
        let _ = config.ensure_bitcoin_conf_exists();
        config
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
