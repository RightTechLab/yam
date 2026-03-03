#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Running,
    Stopped,
    Transitioning,
    Unknown,
}

#[cfg(target_os = "macos")]
pub mod manager {
    use super::*;

    pub async fn check_service_status(name: &str) -> anyhow::Result<NodeStatus> {
        let output = tokio::process::Command::new("brew")
            .args(&["services", "list"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with(name) {
                if line.contains("started") {
                    return Ok(NodeStatus::Running);
                } else if line.contains("stopped") || line.contains("none") || line.contains("error") {
                    return Ok(NodeStatus::Stopped);
                }
            }
        }
        Ok(NodeStatus::Unknown)
    }

    pub async fn mod_service(name: &str, action: &str) -> anyhow::Result<()> {
        tokio::process::Command::new("brew")
            .args(&["services", action, name])
            .output()
            .await?;
        Ok(())
    }

    // Legacy wrappers for compatibility
    pub async fn check_status() -> anyhow::Result<NodeStatus> { check_service_status("bitcoin").await }
    pub async fn check_tor_status() -> anyhow::Result<NodeStatus> { check_service_status("tor").await }
    pub async fn start_node() -> anyhow::Result<()> { mod_service("bitcoin", "start").await }
    pub async fn stop_node() -> anyhow::Result<()> { mod_service("bitcoin", "stop").await }
    pub async fn restart_node() -> anyhow::Result<()> { mod_service("bitcoin", "restart").await }
    pub async fn start_tor() -> anyhow::Result<()> { mod_service("tor", "start").await }
    pub async fn stop_tor() -> anyhow::Result<()> { mod_service("tor", "stop").await }
    pub async fn restart_tor() -> anyhow::Result<()> { mod_service("tor", "restart").await }
}

#[cfg(target_os = "linux")]
pub mod manager {
    use super::*;

    pub async fn check_service_status(name: &str) -> anyhow::Result<NodeStatus> {
        // bitcoind vs bitcoin etc mapping
        let service_name = match name {
            "bitcoin" => "bitcoind",
            _ => name,
        };

        let output = tokio::process::Command::new("systemctl")
            .args(&["is-active", service_name])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim() == "active" {
            Ok(NodeStatus::Running)
        } else {
            Ok(NodeStatus::Stopped)
        }
    }

    pub async fn mod_service(name: &str, action: &str) -> anyhow::Result<()> {
        let service_name = match name {
            "bitcoin" => "bitcoind",
            _ => name,
        };
        tokio::process::Command::new("sudo")
            .args(&["systemctl", action, service_name])
            .output()
            .await?;
        Ok(())
    }

    // Legacy wrappers for compatibility
    pub async fn check_status() -> anyhow::Result<NodeStatus> { check_service_status("bitcoin").await }
    pub async fn check_tor_status() -> anyhow::Result<NodeStatus> { check_service_status("tor").await }
    pub async fn start_node() -> anyhow::Result<()> { mod_service("bitcoin", "start").await }
    pub async fn stop_node() -> anyhow::Result<()> { mod_service("bitcoin", "stop").await }
    pub async fn restart_node() -> anyhow::Result<()> { mod_service("bitcoin", "restart").await }
    pub async fn start_tor() -> anyhow::Result<()> { mod_service("tor", "start").await }
    pub async fn stop_tor() -> anyhow::Result<()> { mod_service("tor", "stop").await }
    pub async fn restart_tor() -> anyhow::Result<()> { mod_service("tor", "restart").await }
}
