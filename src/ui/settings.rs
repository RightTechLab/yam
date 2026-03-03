use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};
use crate::app::{App, SettingsTab};

pub fn render_settings(f: &mut Frame, app: &App) {
    let area = f.area();

    // Create a centered popup
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(popup_layout[1])[1];

    // Clear background
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(5),    // Content
            Constraint::Length(1), // Help Footer
        ])
        .split(inner_area);

    // --- TABS ---
    let tab_titles = vec![" RPC Connection ", " Manage Services "];
    let selected_tab = if app.settings_tab == SettingsTab::Rpc { 0 } else { 1 };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // --- CONTENT ---
    match app.settings_tab {
        SettingsTab::Rpc => {
            let rpc_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Host
                    Constraint::Length(3), // User
                    Constraint::Length(3), // Pass
                    Constraint::Min(1),
                ])
                .split(chunks[1]);

            // Host input
            let host_style = if app.active_input_index == 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let host_input = Paragraph::new(app.rpc_host.as_str())
                .style(host_style)
                .block(Block::default().title(" RPC Host ").borders(Borders::ALL));
            f.render_widget(host_input, rpc_chunks[0]);

            // User input
            let user_style = if app.active_input_index == 1 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let user_input = Paragraph::new(app.rpc_user.as_str())
                .style(user_style)
                .block(Block::default().title(" RPC User ").borders(Borders::ALL));
            f.render_widget(user_input, rpc_chunks[1]);

            // Pass input
            let pass_style = if app.active_input_index == 2 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let pass_display = "*".repeat(app.rpc_pass.len());
            let pass_input = Paragraph::new(pass_display.as_str())
                .style(pass_style)
                .block(Block::default().title(" RPC Password ").borders(Borders::ALL));
            f.render_widget(pass_input, rpc_chunks[2]);
            
            let help = Paragraph::new(" [Tab] Change Focus | [Enter] Apply Settings | [Esc] Close Settings ")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(help, chunks[2]);
        }
        SettingsTab::Services => {
            let svc_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Top padding
                    Constraint::Length(1), // Bitcoin Row
                    Constraint::Length(1), // Tor Row
                    Constraint::Min(1),    // Bottom space
                    Constraint::Length(1), // Help Footer
                ])
                .split(chunks[1]);

            let render_row = |f: &mut Frame, area: Rect, label: &str, service_idx: usize, app: &App| {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(12), Constraint::Min(1)])
                    .split(area);

                f.render_widget(Paragraph::new(label).style(Style::default().fg(Color::Cyan)), row_chunks[0]);

                let actions = ["START", "STOP", "RESTART"];
                let mut spans = Vec::new();

                for (action_idx, action) in actions.iter().enumerate() {
                    let mut style = Style::default().fg(Color::DarkGray);
                    
                    // Highlight if this is the currently selected service AND action
                    if app.selected_service_index == service_idx && app.selected_action_index == action_idx {
                        style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::REVERSED);
                    }
                    
                    spans.push(Span::styled(format!(" {} ", action), style));
                    
                    if action_idx < actions.len() - 1 {
                        spans.push(Span::raw(" | "));
                    }
                }

                f.render_widget(Paragraph::new(Line::from(spans)), row_chunks[1]);
            };

            render_row(f, svc_chunks[1], "Bitcoin:", 0, app);
            render_row(f, svc_chunks[2], "Tor:", 1, app);

            let svc_help = Paragraph::new(" [Up/Down] Select Service  |  [Left/Right] Select Action  |  [Enter] Execute  |  [Tab] Back to RPC  |  [Esc] Close ")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(svc_help, svc_chunks[4]);
        }
    }
}
