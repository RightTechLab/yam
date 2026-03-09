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
        let output = tokio::process::Command::new("brew")
            .args(&["services", action, name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("brew services {} {} failed: {}", action, name, stderr.trim());
        }
        Ok(())
    }

    pub fn sudoers_hint() -> String {
        "On macOS, brew services does not require sudo.".into()
    }
}

#[cfg(target_os = "linux")]
pub mod manager {
    use super::*;

    pub async fn check_service_status(name: &str) -> anyhow::Result<NodeStatus> {
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
        let output = tokio::process::Command::new("sudo")
            .args(&["-n", "systemctl", action, service_name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("password is required") || stderr.contains("sudo") {
                anyhow::bail!(
                    "sudo requires password. Run:\n  sudo visudo -f /etc/sudoers.d/yam\nand add:\n  {} ALL=(ALL) NOPASSWD: /usr/bin/systemctl start {svc}, /usr/bin/systemctl stop {svc}, /usr/bin/systemctl restart {svc}",
                    std::env::var("USER").unwrap_or_else(|_| "youruser".into()),
                    svc = service_name
                );
            }
            anyhow::bail!("systemctl {} {} failed: {}", action, service_name, stderr.trim());
        }
        Ok(())
    }

    pub fn sudoers_hint() -> String {
        let user = std::env::var("USER").unwrap_or_else(|_| "youruser".into());
        let services = ["bitcoind", "tor", "electrs", "i2pd", "btc-rpc-explorer"];
        let mut lines = vec![format!("# /etc/sudoers.d/yam — run: sudo visudo -f /etc/sudoers.d/yam")];
        for svc in &services {
            lines.push(format!(
                "{} ALL=(ALL) NOPASSWD: /usr/bin/systemctl start {svc}, /usr/bin/systemctl stop {svc}, /usr/bin/systemctl restart {svc}",
                user, svc = svc
            ));
        }
        lines.join("\n")
    }
}

