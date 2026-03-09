use std::io;
use std::time::Duration;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::time;
use tokio::sync::Mutex;
use std::sync::Arc;
use futures::StreamExt;
use bitcoincore_rpc::Auth;
use bitcoincore_rpc::json::{GetBlockchainInfoResult, GetMempoolInfoResult};
use chrono::Local;

#[derive(Default)]
struct RpcState {
    chain_info: Option<GetBlockchainInfoResult>,
    mempool_info: Option<GetMempoolInfoResult>,
    network_info: Option<bitcoincore_rpc::json::GetNetworkInfoResult>,
    peer_count: Option<u64>,
    node_status: String,
}

struct SystemState {
    uptime: u64,
    cpu_usage: f32,
    mem_used: u64,
    mem_total: u64,
    disk_used: u64,
    disk_total: u64,
    temp: f32,
    hostname: String,
}

mod app;
mod infra;
mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let config = infra::config::YamConfig::load();
    let mut app = app::App::new(&config);
    
    let mut tick_rate = time::interval(Duration::from_millis(250)); // Faster tick for better UI responsiveness
    let mut event_stream = crossterm::event::EventStream::new();

    // Shared state for background tasks
    let bg_bitcoin_status = Arc::new(Mutex::new("Unknown".to_string()));
    let bg_tor_status = Arc::new(Mutex::new("Unknown".to_string()));
    let bg_electrs_status = Arc::new(Mutex::new("Unknown".to_string()));
    let bg_i2p_status = Arc::new(Mutex::new("Unknown".to_string()));
    let bg_explorer_status = Arc::new(Mutex::new("Unknown".to_string()));
    let bg_logs = Arc::new(Mutex::new(Vec::new()));
    
    let bg_system_state = Arc::new(Mutex::new(SystemState {
        uptime: 0,
        cpu_usage: 0.0,
        mem_used: 0,
        mem_total: 0,
        disk_used: 0,
        disk_total: 0,
        temp: 0.0,
        hostname: "Unknown".into(),
    }));
    
    let bg_rpc_state = Arc::new(Mutex::new(RpcState {
        chain_info: None,
        mempool_info: None,
        peer_count: None,
        network_info: None,
        node_status: "Initializing...".to_string(),
    }));
    let bg_rpc_credentials = Arc::new(Mutex::new((
        app.rpc_host.clone(),
        app.rpc_user.clone(),
        app.rpc_pass.clone(),
    )));

    // Shared state for service action results
    let bg_service_action_result: Arc<Mutex<Option<Result<String, String>>>> = Arc::new(Mutex::new(None));

    // Background task to check services periodically
    let b1 = Arc::clone(&bg_bitcoin_status);
    let t1 = Arc::clone(&bg_tor_status);
    let e1 = Arc::clone(&bg_electrs_status);
    let i1 = Arc::clone(&bg_i2p_status);
    let ex1 = Arc::clone(&bg_explorer_status);
    let l1 = Arc::clone(&bg_logs);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            
            // Bitcoin
            if let Ok(status) = infra::service::manager::check_service_status("bitcoin").await {
                *b1.lock().await = format!("{:?}", status);
            }
            // Tor
            if let Ok(status) = infra::service::manager::check_service_status("tor").await {
                *t1.lock().await = format!("{:?}", status);
            }
            // Electrs
            if let Ok(status) = infra::service::manager::check_service_status("electrs").await {
                *e1.lock().await = format!("{:?}", status);
            }
            // I2P (common names: i2pd, i2p)
            if let Ok(status) = infra::service::manager::check_service_status("i2pd").await {
                *i1.lock().await = format!("{:?}", status);
            } else if let Ok(status) = infra::service::manager::check_service_status("i2p").await {
                *i1.lock().await = format!("{:?}", status);
            }
            // Explorer
            if let Ok(status) = infra::service::manager::check_service_status("btc-rpc-explorer").await {
                *ex1.lock().await = format!("{:?}", status);
            }
        }
    });

    // Background task for system metrics
    let sys_st = Arc::clone(&bg_system_state);
    tokio::spawn(async move {
        use sysinfo::{System, Disks, Components};
        let mut sys = System::new_all();
        let mut interval = time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            sys.refresh_all();
            
            let uptime = System::uptime();
            let cpu = sys.global_cpu_usage();
            let mem_total = sys.total_memory();
            let mem_used = sys.used_memory();
            let hostname = System::host_name().unwrap_or_else(|| "Unknown".into());
            
            // Disk
            let disks = Disks::new_with_refreshed_list();
            let (d_total, d_used) = disks.iter().fold((0, 0), |(t, u), d| {
                (t + d.total_space(), u + (d.total_space() - d.available_space()))
            });

            // Temperature (if available)
            let comps = Components::new_with_refreshed_list();
            let temp = comps.iter()
                .find(|c| c.label().to_lowercase().contains("cpu"))
                .and_then(|c| c.temperature())
                .unwrap_or(0.0);

            let mut guard = sys_st.lock().await;
            guard.uptime = uptime;
            guard.cpu_usage = cpu;
            guard.mem_total = mem_total;
            guard.mem_used = mem_used;
            guard.disk_total = d_total;
            guard.disk_used = d_used;
            guard.hostname = hostname;
            guard.temp = temp;
        }
    });

    // Background task for Tor Hidden Service Hostnames (per-service)
    let bg_detected_onion = Arc::new(Mutex::new(None::<String>));
    let bg_detected_electrs_onion = Arc::new(Mutex::new(None::<String>));
    let bg_detected_mempool_onion = Arc::new(Mutex::new(None::<String>));
    let bg_detected_explorer_onion = Arc::new(Mutex::new(None::<String>));
    let detected_onion = Arc::clone(&bg_detected_onion);
    let detected_electrs = Arc::clone(&bg_detected_electrs_onion);
    let detected_mempool = Arc::clone(&bg_detected_mempool_onion);
    let detected_explorer = Arc::clone(&bg_detected_explorer_onion);
    tokio::spawn(async move {
        let bitcoin_paths = [
            "/var/lib/tor/bitcoinrpc/hostname",
        ];
        let electrs_paths = [
            "/var/lib/tor/electrs/hostname",
        ];
        let mempool_paths = [
            "/var/lib/tor/mempool/hostname",
        ];
        let explorer_paths = [
            "/var/lib/tor/bitcoinexplorer/hostname",
        ];

        async fn read_onion(paths: &[&str]) -> Option<String> {
            for path in paths {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    let onion = content.trim().to_string();
                    if !onion.is_empty() {
                        return Some(onion);
                    }
                }
            }
            None
        }

        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Some(onion) = read_onion(&bitcoin_paths).await {
                *detected_onion.lock().await = Some(onion);
            }
            if let Some(onion) = read_onion(&electrs_paths).await {
                *detected_electrs.lock().await = Some(onion);
            }
            if let Some(onion) = read_onion(&mempool_paths).await {
                *detected_mempool.lock().await = Some(onion);
            }
            if let Some(onion) = read_onion(&explorer_paths).await {
                *detected_explorer.lock().await = Some(onion);
            }
        }
    });

    // Background task to poll RPC data
    let rpc_st = Arc::clone(&bg_rpc_state);
    let rpc_cred = Arc::clone(&bg_rpc_credentials);
    let l2 = Arc::clone(&bg_logs);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;

            let (host, user, pass) = {
                let guard = rpc_cred.lock().await;
                (guard.0.clone(), guard.1.clone(), guard.2.clone())
            };
            
            l2.lock().await.push(format!("Polling RPC at {}...", host));

            let result = tokio::task::spawn_blocking(move || {
                let auth = Auth::UserPass(user, pass);
                if let Ok(rpc) = infra::rpc::RpcClient::with_auth(&host, auth) {
                    let chain = rpc.get_chain_info().ok();
                    let mempool = rpc.get_mempool_info().ok();
                    let network = rpc.get_network_info().ok();
                    let peers = rpc.get_peer_count().ok();
                    
                    let status = if chain.is_some() {
                        "Connected".to_string()
                    } else {
                        "Connected (Waiting for data...)".to_string()
                    };
                    
                    Some((chain, mempool, network, peers, status))
                } else {
                    None
                }
            }).await;

            let mut state = rpc_st.lock().await;
            if let Ok(Some((chain, mempool, network, peers, status))) = result {
                state.chain_info = chain;
                state.mempool_info = mempool;
                state.network_info = network;
                state.peer_count = peers;
                state.node_status = status;
            } else {
                state.node_status = "Connection Failed".to_string();
                state.chain_info = None;
                state.mempool_info = None;
                state.network_info = None;
                state.peer_count = None;
            }
        }
    });

    loop {
        terminal.draw(|f| {
            ui::dashboard::render_dashboard(f, &app);

            // If in settings mode, render the popup on top
            if app.mode == app::AppMode::Settings {
                ui::settings::render_settings(f, &app);
            } else if app.mode == app::AppMode::Playground {
                ui::playground::render_playground(f, &app);
            }
        })?;

        tokio::select! {
            _ = tick_rate.tick() => {
                // Spinner tick + check for completed service actions
                if app.service_action_busy {
                    app.spinner_tick = app.spinner_tick.wrapping_add(1);
                    if let Some(result) = bg_service_action_result.lock().await.take() {
                        app.service_action_busy = false;
                        match result {
                            Ok(msg) => {
                                app.add_log(msg);
                                app.status_message = "Done".into();
                            }
                            Err(msg) => {
                                app.add_log(msg.clone());
                                app.status_message = msg;
                            }
                        }
                    }
                }
                {
                    app.bitcoin_service_status = bg_bitcoin_status.lock().await.clone();
                    app.tor_service_status = bg_tor_status.lock().await.clone();
                    app.electrs_service_status = bg_electrs_status.lock().await.clone();
                    app.i2p_service_status = bg_i2p_status.lock().await.clone();
                    app.explorer_service_status = bg_explorer_status.lock().await.clone();
                }
                {
                    let sys = bg_system_state.lock().await;
                    app.node_hostname = sys.hostname.clone();
                    app.uptime = format!("up {}h {}m", sys.uptime / 3600, (sys.uptime % 3600) / 60);
                    app.cpu_load = format!("{:.2}%", sys.cpu_usage);
                    // sysinfo returns bytes for total_memory/used_memory
                    app.memory_info = format!("{:.1}Gi / {:.1}Gi", sys.mem_used as f64 / 1024.0 / 1024.0 / 1024.0, sys.mem_total as f64 / 1024.0 / 1024.0 / 1024.0);
                    app.disk_info = format!("{:.1}Ti / {:.1}Ti", sys.disk_used as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0, sys.disk_total as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0);
                    app.cpu_temp = format!("{:.1}°C", sys.temp);
                }
                
                // Fetch Local IP if not already set
                if app.local_ip == "Determining..." {
                    if let Ok(output) = std::process::Command::new("hostname").arg("-I").output() {
                        let all_ips = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if let Some(first_ip) = all_ips.split_whitespace().next() {
                            app.local_ip = first_ip.to_string();
                        }
                    }
                }

                {
                    let r_guard = bg_rpc_state.lock().await;
                    app.chain_info = r_guard.chain_info.clone();
                    app.mempool_info = r_guard.mempool_info.clone();
                    app.network_info = r_guard.network_info.clone();
                    app.peer_count = r_guard.peer_count;
                    
                    if let Some(ref chain) = r_guard.chain_info {
                        app.node_network = chain.chain.to_string();
                    }

                    if let Some(ref ni) = r_guard.network_info {
                        let v = ni.version;
                        app.node_version = format!("{}.{}.{}", v / 10000, (v % 10000) / 100, v % 100);
                        
                        for network in &ni.networks {
                            match network.name.as_str() {
                                "onion" => {
                                    if network.reachable {
                                        if let Some(la) = ni.local_addresses.iter().find(|la| la.address.contains(".onion")) {
                                            app.tor_onion = la.address.clone();
                                        } else {
                                            // Fallback to detected onion if available
                                            if let Some(detected) = bg_detected_onion.lock().await.clone() {
                                                app.tor_onion = detected;
                                            } else {
                                                app.tor_onion = "Reachable".into();
                                            }
                                        }
                                        // Per-service onion addresses (fall back to bitcoin onion)
                                        if let Some(detected) = bg_detected_electrs_onion.lock().await.clone() {
                                            app.electrs_onion = detected;
                                        } else {
                                            app.electrs_onion = app.tor_onion.clone();
                                        }
                                        if let Some(detected) = bg_detected_mempool_onion.lock().await.clone() {
                                            app.mempool_onion = detected;
                                        } else {
                                            app.mempool_onion = app.tor_onion.clone();
                                        }
                                        if let Some(detected) = bg_detected_explorer_onion.lock().await.clone() {
                                            app.explorer_onion = detected;
                                        } else {
                                            app.explorer_onion = app.tor_onion.clone();
                                        }
                                    } else {
                                        app.tor_onion = "Disabled".into();
                                        app.electrs_onion = "Disabled".into();
                                        app.mempool_onion = "Disabled".into();
                                        app.explorer_onion = "Disabled".into();
                                    }
                                }
                                "i2p" => {
                                    app.i2p_addr = if network.reachable { "Reachable".into() } else { "Disabled".into() };
                                }
                                _ => {}
                            }
                        }
                    }

                    if app.node_status != r_guard.node_status {
                        app.add_log(format!("RPC Status changed to: {}", r_guard.node_status));
                    }
                    app.node_status = r_guard.node_status.clone();
                }
                {
                    let mut l_guard = bg_logs.lock().await;
                    for log in l_guard.drain(..) {
                        app.add_log(log);
                    }
                }
                app.current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            }
            Some(Ok(Event::Key(key))) = event_stream.next() => {
                // Global quit on Ctrl-C
                if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    break;
                }

                if app.mode == app::AppMode::Dashboard {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('i') => {
                            app.show_logs = !app.show_logs;
                        }
                        KeyCode::Char('s') => {
                            app.mode = app::AppMode::Settings;
                            app.status_message = "Entered Settings".into();
                        }
                        KeyCode::Char('p') => {
                            app.mode = app::AppMode::Playground;
                            app.status_message = "Entered Playground".into();
                        }
                        _ => {}
                    }
                } else if app.mode == app::AppMode::Settings {
                    match key.code {
                        KeyCode::Esc => {
                            app.mode = app::AppMode::Dashboard;
                            app.status_message = "Settings cancelled".into();
                        }
                        KeyCode::Tab => {
                            app.settings_tab = match app.settings_tab {
                                app::SettingsTab::Rpc => app::SettingsTab::Services,
                                app::SettingsTab::Services => {
                                    app.load_bitcoin_conf();
                                    app::SettingsTab::BitcoinConf
                                }
                                app::SettingsTab::BitcoinConf => app::SettingsTab::Rpc,
                            };
                        }
                        _ => {
                            // Tab-specific key handlers
                            match app.settings_tab {
                                app::SettingsTab::Rpc => {
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.mode = app::AppMode::Dashboard;
                                            app.status_message = "Applying RPC Settings...".into();
                                            app.node_status = "Applying...".into();
                                            
                                            // Save to disk
                                            let save_config = infra::config::YamConfig {
                                                rpc_host: app.rpc_host.clone(),
                                                rpc_user: app.rpc_user.clone(),
                                                rpc_pass: app.rpc_pass.clone(),
                                            };
                                            if let Err(e) = save_config.save() {
                                                app.add_log(format!("Failed to save config: {}", e));
                                            } else {
                                                app.add_log("Config saved to ~/.yam/config.toml".into());
                                            }
                                            
                                            // Update in-memory credentials for background RPC poller
                                            let rpc_cred = Arc::clone(&bg_rpc_credentials);
                                            let host = app.rpc_host.clone();
                                            let user = app.rpc_user.clone();
                                            let pass = app.rpc_pass.clone();
                                            tokio::spawn(async move {
                                                let mut guard = rpc_cred.lock().await;
                                                *guard = (host, user, pass);
                                            });
                                        }
                                        KeyCode::Backspace => {
                                            match app.active_input_index {
                                                0 => { app.rpc_host.pop(); }
                                                1 => { app.rpc_user.pop(); }
                                                2 => { app.rpc_pass.pop(); }
                                                _ => {}
                                            }
                                        }
                                        KeyCode::Char(c) => {
                                            match app.active_input_index {
                                                0 => { app.rpc_host.push(c); }
                                                1 => { app.rpc_user.push(c); }
                                                2 => { app.rpc_pass.push(c); }
                                                _ => {}
                                            }
                                        }
                                        KeyCode::Down => {
                                            app.active_input_index = (app.active_input_index + 1) % 3;
                                        }
                                        KeyCode::Up => {
                                            if app.active_input_index == 0 {
                                                app.active_input_index = 2;
                                            } else {
                                                app.active_input_index -= 1;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                app::SettingsTab::Services => {
                                    match key.code {
                                        KeyCode::Up => {
                                            if app.selected_service_index == 0 {
                                                app.selected_service_index = 4;
                                            } else {
                                                app.selected_service_index -= 1;
                                            }
                                        }
                                        KeyCode::Down => {
                                            app.selected_service_index = (app.selected_service_index + 1) % 5;
                                        }
                                        KeyCode::Left => {
                                            if app.selected_action_index > 0 {
                                                app.selected_action_index -= 1;
                                            } else {
                                                app.selected_action_index = 2;
                                            }
                                        }
                                        KeyCode::Right => {
                                            app.selected_action_index = (app.selected_action_index + 1) % 3;
                                        }
                                        KeyCode::Enter => {
                                            let (service_label, service_name) = match app.selected_service_index {
                                                0 => ("Bitcoin Node", "bitcoin"),
                                                1 => ("Tor", "tor"),
                                                2 => ("Electrs", "electrs"),
                                                3 => ("I2P", "i2pd"),
                                                4 => ("BTC RPC Explorer", "btc-rpc-explorer"),
                                                _ => ("Unknown", "unknown"),
                                            };
                                            let action = match app.selected_action_index {
                                                0 => "start",
                                                1 => "stop",
                                                2 => "restart",
                                                _ => "unknown",
                                            };
                                            let action_label = match app.selected_action_index {
                                                0 => "Starting",
                                                1 => "Stopping",
                                                2 => "Restarting",
                                                _ => "Unknown",
                                            };
                                            
                                            if !app.service_action_busy {
                                                app.service_action_busy = true;
                                                app.spinner_tick = 0;
                                                app.service_action_label = format!("{} {}...", action_label, service_label);
                                                app.add_log(app.service_action_label.clone());
                                                app.status_message = app.service_action_label.clone();

                                                let result_arc = Arc::clone(&bg_service_action_result);
                                                let svc = service_name.to_string();
                                                let act = action.to_string();
                                                let label = service_label.to_string();
                                                let action_l = action_label.to_string();
                                                tokio::spawn(async move {
                                                    let result = match infra::service::manager::mod_service(&svc, &act).await {
                                                        Ok(()) => Ok(format!("{} {} done.", action_l, label)),
                                                        Err(e) => Err(format!("Failed: {}", e)),
                                                    };
                                                    *result_arc.lock().await = Some(result);
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                app::SettingsTab::BitcoinConf => {
                                    match key.code {
                                        KeyCode::Up => {
                                            if app.conf_cursor_y > 0 {
                                                app.conf_cursor_y -= 1;
                                                let line_len = app.conf_lines[app.conf_cursor_y].len();
                                                if app.conf_cursor_x > line_len {
                                                    app.conf_cursor_x = line_len;
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            if app.conf_cursor_y < app.conf_lines.len() - 1 {
                                                app.conf_cursor_y += 1;
                                                let line_len = app.conf_lines[app.conf_cursor_y].len();
                                                if app.conf_cursor_x > line_len {
                                                    app.conf_cursor_x = line_len;
                                                }
                                            }
                                        }
                                        KeyCode::Left => {
                                            if app.conf_cursor_x > 0 {
                                                app.conf_cursor_x -= 1;
                                            } else if app.conf_cursor_y > 0 {
                                                app.conf_cursor_y -= 1;
                                                app.conf_cursor_x = app.conf_lines[app.conf_cursor_y].len();
                                            }
                                        }
                                        KeyCode::Right => {
                                            let line_len = app.conf_lines[app.conf_cursor_y].len();
                                            if app.conf_cursor_x < line_len {
                                                app.conf_cursor_x += 1;
                                            } else if app.conf_cursor_y < app.conf_lines.len() - 1 {
                                                app.conf_cursor_y += 1;
                                                app.conf_cursor_x = 0;
                                            }
                                        }
                                        KeyCode::Home => {
                                            app.conf_cursor_x = 0;
                                        }
                                        KeyCode::End => {
                                            app.conf_cursor_x = app.conf_lines[app.conf_cursor_y].len();
                                        }
                                        KeyCode::Char('s') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                            match app.save_bitcoin_conf() {
                                                Ok(()) => {
                                                    app.add_log("bitcoin.conf saved.".into());
                                                    app.status_message = "bitcoin.conf saved!".into();
                                                }
                                                Err(e) => {
                                                    app.add_log(format!("Failed to save bitcoin.conf: {}", e));
                                                    app.status_message = format!("Save failed: {}", e);
                                                }
                                            }
                                        }
                                        KeyCode::Char(c) => {
                                            app.conf_lines[app.conf_cursor_y].insert(app.conf_cursor_x, c);
                                            app.conf_cursor_x += 1;
                                            app.conf_dirty = true;
                                        }
                                        KeyCode::Backspace => {
                                            if app.conf_cursor_x > 0 {
                                                app.conf_lines[app.conf_cursor_y].remove(app.conf_cursor_x - 1);
                                                app.conf_cursor_x -= 1;
                                                app.conf_dirty = true;
                                            } else if app.conf_cursor_y > 0 {
                                                let removed = app.conf_lines.remove(app.conf_cursor_y);
                                                app.conf_cursor_y -= 1;
                                                app.conf_cursor_x = app.conf_lines[app.conf_cursor_y].len();
                                                app.conf_lines[app.conf_cursor_y].push_str(&removed);
                                                app.conf_dirty = true;
                                            }
                                        }
                                        KeyCode::Delete => {
                                            let line_len = app.conf_lines[app.conf_cursor_y].len();
                                            if app.conf_cursor_x < line_len {
                                                app.conf_lines[app.conf_cursor_y].remove(app.conf_cursor_x);
                                                app.conf_dirty = true;
                                            } else if app.conf_cursor_y < app.conf_lines.len() - 1 {
                                                let next = app.conf_lines.remove(app.conf_cursor_y + 1);
                                                app.conf_lines[app.conf_cursor_y].push_str(&next);
                                                app.conf_dirty = true;
                                            }
                                        }
                                        KeyCode::Enter => {
                                            let rest = app.conf_lines[app.conf_cursor_y].split_off(app.conf_cursor_x);
                                            app.conf_cursor_y += 1;
                                            app.conf_lines.insert(app.conf_cursor_y, rest);
                                            app.conf_cursor_x = 0;
                                            app.conf_dirty = true;
                                        }
                                        _ => {}
                                    }
                                    // Keep scroll in sync with cursor
                                    let visible_height = 15usize;
                                    if app.conf_cursor_y < app.conf_scroll {
                                        app.conf_scroll = app.conf_cursor_y;
                                    } else if app.conf_cursor_y >= app.conf_scroll + visible_height {
                                        app.conf_scroll = app.conf_cursor_y - visible_height + 1;
                                    }
                                }
                            }
                        }
                    }
                } else if app.mode == app::AppMode::Playground {
                    let update_suggestions = |app: &mut App| {
                        let input = app.playground_input.trim_start();
                        if input.is_empty() {
                            app.playground_suggestions.clear();
                            app.playground_suggestion_idx = None;
                            return;
                        }
                        
                        let last_word = input.split_whitespace().last().unwrap_or("");
                        
                        if last_word.is_empty() {
                            app.playground_suggestions.clear();
                            app.playground_suggestion_idx = None;
                            return;
                        }

                        app.playground_suggestions = app.playground_command_list
                            .iter()
                            .filter(|cmd| cmd.starts_with(last_word) && *cmd != &last_word)
                            .map(|s| s.to_string())
                            .collect();
                            
                        if !app.playground_suggestions.is_empty() {
                            app.playground_suggestion_idx = Some(0);
                        } else {
                            app.playground_suggestion_idx = None;
                        }
                    };

                    match key.code {
                        KeyCode::Esc => {
                            app.mode = app::AppMode::Dashboard;
                        }
                        KeyCode::Backspace => {
                            app.playground_input.pop();
                            update_suggestions(&mut app);
                        }
                        KeyCode::Char(c) => {
                            app.playground_input.push(c);
                            update_suggestions(&mut app);
                        }
                        KeyCode::Tab => {
                            if !app.playground_suggestions.is_empty() {
                                if let Some(idx) = app.playground_suggestion_idx {
                                    let suggestion = &app.playground_suggestions[idx];
                                    let mut words: Vec<&str> = app.playground_input.split_whitespace().collect();
                                    if !words.is_empty() {
                                        words.pop();
                                    }
                                    
                                    app.playground_input = words.join(" ");
                                    if !app.playground_input.is_empty() {
                                        app.playground_input.push(' ');
                                    }
                                    app.playground_input.push_str(suggestion);
                                    app.playground_input.push(' ');
                                    
                                    app.playground_suggestions.clear();
                                    app.playground_suggestion_idx = None;
                                }
                            }
                        }
                        KeyCode::Char('l') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            app.playground_history.clear();
                            app.playground_scroll = 0;
                            app.playground_history.push("Playground cleared.".into());
                        }
                        KeyCode::Up if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                            app.playground_scroll = app.playground_scroll.saturating_add(5);
                        }
                        KeyCode::Down if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                            app.playground_scroll = app.playground_scroll.saturating_sub(5);
                        }
                        KeyCode::Up => {
                            if !app.playground_suggestions.is_empty() {
                                if let Some(idx) = app.playground_suggestion_idx {
                                    let len = app.playground_suggestions.len();
                                    app.playground_suggestion_idx = Some(if idx == 0 { len - 1 } else { idx - 1 });
                                }
                            }
                        }
                        KeyCode::Down => {
                            if !app.playground_suggestions.is_empty() {
                                if let Some(idx) = app.playground_suggestion_idx {
                                    app.playground_suggestion_idx = Some((idx + 1) % app.playground_suggestions.len());
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if !app.playground_input.trim().is_empty() {
                                let cmd = app.playground_input.trim().to_string();
                                app.playground_input.clear();
                                app.playground_suggestions.clear();
                                app.playground_suggestion_idx = None;
                                app.playground_scroll = 0;

                                if cmd == "clear" {
                                    app.playground_history.clear();
                                    app.playground_history.push("Playground cleared.".into());
                                } else {
                                    app.playground_history.push(format!("> {}", cmd));
                                    
                                    // Execute user command natively 
                                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                                    let program = parts[0];
                                    let args = &parts[1..];
                                    
                                    // Fire the async shell spawn 
                                    let _ = tokio::process::Command::new(program)
                                        .args(args)
                                        .output()
                                        .await
                                        .map(|output| {
                                            if output.status.success() {
                                                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                                                app.playground_history.push(stdout);
                                            } else {
                                                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                                                app.playground_history.push(format!("Error: {}", stderr));
                                            }
                                        })
                                        .map_err(|e| {
                                            app.playground_history.push(format!("Error executing command: {}", e));
                                        });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}