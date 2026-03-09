use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamConfig {
    pub rpc_host: String,
    pub rpc_user: String,
    pub rpc_pass: String,
}

impl Default for YamConfig {
    fn default() -> Self {
        Self {
            rpc_host: "http://127.0.0.1:18443".into(),
            rpc_user: "bitcoin".into(),
            rpc_pass: "bitcoin".into(),
        }
    }
}

impl YamConfig {
    /// Returns the config file path: ~/.yam/config.toml
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".yam").join("config.toml")
    }

    /// Load config from ~/.yam/config.toml, falling back to defaults if missing
    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save config to ~/.yam/config.toml
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
