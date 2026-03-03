use bitcoincore_rpc::json::{GetBlockchainInfoResult, GetMempoolInfoResult};

#[derive(Default, PartialEq)]
pub enum AppMode {
    #[default]
    Dashboard,
    Settings,
    Playground,
}

#[derive(Default, PartialEq)]
pub enum SettingsTab {
    #[default]
    Rpc,
    Services,
}

#[derive(Default)]
pub struct App {
    pub mode: AppMode,
    pub settings_tab: SettingsTab,
    pub node_status: String,
    pub bitcoin_service_status: String,
    pub tor_service_status: String,
    pub electrs_service_status: String,
    pub i2p_service_status: String,
    pub explorer_service_status: String,
    pub chain_info: Option<GetBlockchainInfoResult>,
    pub mempool_info: Option<GetMempoolInfoResult>,
    pub network_info: Option<bitcoincore_rpc::json::GetNetworkInfoResult>,
    pub peer_count: Option<u64>,
    pub should_quit: bool,
    pub status_message: String,
    
    // System Health
    pub uptime: String,
    pub cpu_load: String,
    pub cpu_temp: String,
    pub memory_info: String,
    pub disk_info: String,
    pub node_hostname: String,
    
    // New network fields
    pub local_ip: String,
    pub node_version: String,
    pub node_network: String,
    pub tor_onion: String,
    pub i2p_addr: String,
    
    // Settings state
    pub rpc_host: String,
    pub rpc_user: String,
    pub rpc_pass: String,
    pub active_input_index: usize, 
    
    // Services tab state
    pub selected_service_index: usize, 
    pub selected_action_index: usize,
    
    // Playground state
    pub playground_input: String,
    pub playground_history: Vec<String>,
    pub playground_command_list: Vec<&'static str>,
    pub playground_suggestions: Vec<String>,
    pub playground_suggestion_idx: Option<usize>,
    pub playground_scroll: usize,
    
    // Activity Logs
    pub logs: Vec<String>,
    pub show_logs: bool,
    pub current_time: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Dashboard,
            settings_tab: SettingsTab::Rpc,
            node_status: "Unknown".to_string(),
            bitcoin_service_status: "Unknown".to_string(),
            tor_service_status: "Unknown".to_string(),
            electrs_service_status: "Unknown".to_string(),
            i2p_service_status: "Unknown".to_string(),
            explorer_service_status: "Unknown".to_string(),
            chain_info: None,
            mempool_info: None,
            network_info: None,
            peer_count: None,
            should_quit: false,
            status_message: "Initializing...".to_string(),
            
            uptime: "N/A".into(),
            cpu_load: "N/A".into(),
            cpu_temp: "N/A".into(),
            memory_info: "N/A".into(),
            disk_info: "N/A".into(),
            node_hostname: "Unknown".into(),
            
            local_ip: "Determining...".into(),
            node_version: "30.2".into(),
            node_network: "Unknown".into(),
            tor_onion: "Unknown".into(),
            i2p_addr: "Disabled".into(),
            
            rpc_host: "http://127.0.0.1:18443".into(),
            rpc_user: "bitcoin".into(),
            rpc_pass: "bitcoin".into(),
            active_input_index: 0,
            
            selected_service_index: 0,
            selected_action_index: 0,
            
            playground_input: String::new(),
            playground_history: vec!["Welcome to Yam Playground!".into(), "Type command and press Enter (e.g. `bitcoin-cli -regtest getblockchaininfo`).".into()],
            
            playground_command_list: vec![
                "getblockchaininfo", "getnetworkinfo", "getmempoolinfo", "getpeerinfo",
                "getmininginfo", "getwalletinfo", "getbalance", "getnewaddress", "getblockcount",
                "getblockhash", "getblock", "getrawtransaction", "sendtoaddress", "generatetoaddress",
                "createwallet", "loadwallet", "unloadwallet", "getrpcinfo", "help", "stop"
            ],
            playground_suggestions: Vec::new(),
            playground_suggestion_idx: None,
            playground_scroll: 0,
            
            logs: vec!["Application started".into()],
            show_logs: false,
            current_time: String::new(),
        }
    }

    pub fn add_log(&mut self, msg: String) {
        self.logs.push(msg);
        if self.logs.len() > 50 {
            self.logs.remove(0);
        }
    }
}