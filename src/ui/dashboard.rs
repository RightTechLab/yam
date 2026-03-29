use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn render_dashboard(f: &mut Frame, app: &App) {
    let area = f.area();

    // Build constraints dynamically based on whether logs are visible
    let mut constraints = vec![
        Constraint::Length(4), // Header: border(2) + content(2)
        Constraint::Min(10),   // Main
    ];
    
    if app.show_logs {
        constraints.push(Constraint::Length(8)); // Logs
    }
    constraints.push(Constraint::Length(1));     // Footer
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    fn status_style(status: &str) -> Style {
        let status = status.to_lowercase();
        let color = if status.contains("running") || status.contains("connected") {
            Color::Green
        } else if status.contains("unknown") || status.contains("waiting") || status.contains("transitioning") {
            Color::Yellow
        } else {
            Color::Red
        };
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }

    // --- HEADER ---
    let title = Paragraph::new(" YAM - โปรเจคยามว่างที่ทำยามเหงา จนยามตกกะจัยมาถึง ")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Split Main Content into Sub-header (Time) and Main Data section
    let main_row_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(7)])
        .split(chunks[1]);

    let last_updated = Paragraph::new(format!("Last Updated: {}", app.current_time))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(last_updated, main_row_layout[0]);

    // Split Data section into Left and Right columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_row_layout[1]);

    // --- LEFT COLUMN: System and Services ---
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // System Health
            Constraint::Min(8),    // Services Status
        ])
        .split(main_chunks[0]);

    // 1. SYSTEM HEALTH
    let sys_health_text = vec![
        Line::from(vec![Span::raw("Uptime:      "), Span::styled(&app.uptime, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("CPU Load:    "), Span::styled(&app.cpu_load, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("CPU Temp:    "), Span::styled(&app.cpu_temp, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Memory:      "), Span::styled(&app.memory_info, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Disk Space:  "), Span::styled(&app.disk_info, Style::default().add_modifier(Modifier::BOLD))]),
    ];
    let sys_block = Paragraph::new(sys_health_text)
        .block(Block::default().title(" SYSTEM HEALTH ").borders(Borders::ALL));
    f.render_widget(sys_block, left_chunks[0]);

    // 2. SERVICES STATUS
    fn service_line<'a>(name: &'a str, status: &'a str) -> Line<'a> {
        let (icon, color) = if status.to_lowercase().contains("running") {
            (" ● ", Color::Green)
        } else if status.to_lowercase().contains("stopped") {
            (" ● ", Color::Red)
        } else {
            (" ● ", Color::Yellow)
        };
        Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::raw(format!("{:<20}", name)),
            Span::styled(status.to_uppercase(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ])
    }

    let services_text = vec![
        service_line("Bitcoin Core", &app.bitcoin_service_status),
        service_line("Electrs", &app.electrs_service_status),
        service_line("Tor Network", &app.tor_service_status),
        service_line("I2P Router", &app.i2p_service_status),
        service_line("BTC RPC Explorer", &app.explorer_service_status),
    ];
    let services_block = Paragraph::new(services_text)
        .block(Block::default().title(" SERVICES STATUS ").borders(Borders::ALL));
    f.render_widget(services_block, left_chunks[1]);

    // --- RIGHT COLUMN: Bitcoin and Local Networks ---
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Bitcoin Network (increased for room)
            Constraint::Length(8), // Local Network
            Constraint::Min(6),    // Tor Hidden Services
        ])
        .split(main_chunks[1]);

    // 3. BITCOIN NETWORK
    let sync_info = if let Some(ref chain) = app.chain_info {
        let progress = chain.verification_progress * 100.0;
        format!("{:.1}% ({} / {})", progress, chain.blocks, chain.headers)
    } else {
        "Waiting...".into()
    };

    let bitcoin_net_text = vec![
        Line::from(vec![Span::raw("Network Type:  "), Span::styled(&app.node_network, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Sync Progress: "), Span::styled(sync_info, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Version:       "), Span::styled(&app.node_version, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Tor Address:   "), Span::styled(&app.tor_onion, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("I2P Address:   "), Span::styled(&app.i2p_addr, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw(format!("Peers:         {} connected", app.peer_count.unwrap_or(0)))]),
    ];
    let bitcoin_net_block = Paragraph::new(bitcoin_net_text)
        .block(Block::default().title(" BITCOIN NETWORK ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(bitcoin_net_block, right_chunks[0]);

    // 4. LOCAL NETWORK
    let local_net_text = vec![
        Line::from(vec![Span::raw("Hostname:      "), Span::styled(&app.node_hostname, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Local IP:      "), Span::styled(&app.local_ip, Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw(format!("Bitcoin RPC:   http://{}:8332", app.local_ip))]),
        Line::from(vec![Span::raw(format!("Electrs:       http://{}:50001", app.local_ip))]),
        Line::from(vec![Span::raw(format!("Mempool:       http://{}:8888", app.local_ip))]),
    ];
    let local_net_block = Paragraph::new(local_net_text)
        .block(Block::default().title(" LOCAL NETWORK ").borders(Borders::ALL));
    f.render_widget(local_net_block, right_chunks[1]);

    // 5. TOR HIDDEN SERVICES
    let has_tor = |s: &str| s != "Disabled" && s != "Unknown" && s != "Determining...";
    let tor_services_text = if has_tor(&app.tor_bitcoin_rpc_onion) || has_tor(&app.electrs_onion) || has_tor(&app.mempool_onion) || has_tor(&app.explorer_onion) {
        vec![
            Line::from(vec![Span::styled(" ● ", Style::default().fg(Color::Green)), Span::raw(format!("{:<15} {}:8332", "Bitcoin RPC", app.tor_bitcoin_rpc_onion))]),
            Line::from(vec![Span::styled(" ● ", Style::default().fg(Color::Green)), Span::raw(format!("{:<15} {}:50001", "Electrs", app.electrs_onion))]),
            Line::from(vec![Span::styled(" ● ", Style::default().fg(Color::Green)), Span::raw(format!("{:<15} {}:8888", "Mempool", app.mempool_onion))]),
            Line::from(vec![Span::styled(" ● ", Style::default().fg(Color::Green)), Span::raw(format!("{:<15} {}:3002", "BTC Explorer", app.explorer_onion))]),
        ]
    } else {
        vec![Line::from("Tor Hidden Services are not active.")]
    };
    let tor_services_block = Paragraph::new(tor_services_text)
        .block(Block::default().title(" TOR HIDDEN SERVICES ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(tor_services_block, right_chunks[2]);


    // --- LOGS ---
    let footer_idx = if app.show_logs {
        // Take the last 6 logs and preserve their order
        let log_lines: Vec<Line> = app.logs.iter().rev().take(6).rev().map(|l| Line::from(l.as_str())).collect();
        
        let logs_block = Paragraph::new(log_lines)
            .block(Block::default().title(" Activity Logs ").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(logs_block, chunks[2]);
        3
    } else {
        2
    };

    // --- FOOTER: CONTROLS & LOGS ---
    let footer_text = if app.show_logs {
        format!("[s] Settings  [i] Hide Logs  [q] Quit [p] playground mode | Msg: {}", app.status_message)
    } else {
        format!("[s] Settings  [i] Show Logs  [q] Quit [p] playground mode | Msg: {}", app.status_message)
    };
    
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[footer_idx]);
}
