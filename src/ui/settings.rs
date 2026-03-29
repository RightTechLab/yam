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
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(8),
            Constraint::Percentage(84),
            Constraint::Percentage(8),
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
    let tab_titles = vec![" RPC Connection ", " Manage Services ", " Bitcoin.conf "];
    let selected_tab = match app.settings_tab {
        SettingsTab::Rpc => 0,
        SettingsTab::Services => 1,
        SettingsTab::BitcoinConf => 2,
    };

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
                    Constraint::Length(3), // Tor Bitcoin
                    Constraint::Length(3), // Tor Electrs
                    Constraint::Length(3), // Tor Mempool
                    Constraint::Length(3), // Tor Explorer
                    Constraint::Min(1),
                ])
                .split(chunks[1]);

            let input_style = |idx: usize| -> Style {
                if app.active_input_index == idx {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                }
            };

            // Host input
            let host_input = Paragraph::new(app.rpc_host.as_str())
                .style(input_style(0))
                .block(Block::default().title(" RPC Host ").borders(Borders::ALL));
            f.render_widget(host_input, rpc_chunks[0]);

            // User input
            let user_input = Paragraph::new(app.rpc_user.as_str())
                .style(input_style(1))
                .block(Block::default().title(" RPC User ").borders(Borders::ALL));
            f.render_widget(user_input, rpc_chunks[1]);

            // Pass input
            let pass_display = "*".repeat(app.rpc_pass.len());
            let pass_input = Paragraph::new(pass_display.as_str())
                .style(input_style(2))
                .block(Block::default().title(" RPC Password ").borders(Borders::ALL));
            f.render_widget(pass_input, rpc_chunks[2]);

            // Tor Bitcoin hostname path
            let tor_btc_input = Paragraph::new(app.tor_bitcoin_hostname_path.as_str())
                .style(input_style(3))
                .block(Block::default().title(" Tor Bitcoin Hostname Path ").borders(Borders::ALL));
            f.render_widget(tor_btc_input, rpc_chunks[3]);

            // Tor Electrs hostname path
            let tor_electrs_input = Paragraph::new(app.tor_electrs_hostname_path.as_str())
                .style(input_style(4))
                .block(Block::default().title(" Tor Electrs Hostname Path ").borders(Borders::ALL));
            f.render_widget(tor_electrs_input, rpc_chunks[4]);

            // Tor Mempool hostname path
            let tor_mempool_input = Paragraph::new(app.tor_mempool_hostname_path.as_str())
                .style(input_style(5))
                .block(Block::default().title(" Tor Mempool Hostname Path ").borders(Borders::ALL));
            f.render_widget(tor_mempool_input, rpc_chunks[5]);

            // Tor Explorer hostname path
            let tor_explorer_input = Paragraph::new(app.tor_explorer_hostname_path.as_str())
                .style(input_style(6))
                .block(Block::default().title(" Tor Explorer Hostname Path ").borders(Borders::ALL));
            f.render_widget(tor_explorer_input, rpc_chunks[6]);
            
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
                    Constraint::Length(1), // Electrs Row
                    Constraint::Length(1), // I2P Row
                    Constraint::Length(1), // BTC Explorer Row
                    Constraint::Min(1),    // Bottom space
                    Constraint::Length(1), // Help Footer
                ])
                .split(chunks[1]);

            let render_row = |f: &mut Frame, area: Rect, label: &str, service_idx: usize, app: &App| {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(16), Constraint::Min(1)])
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
            render_row(f, svc_chunks[3], "Electrs:", 2, app);
            render_row(f, svc_chunks[4], "I2P:", 3, app);
            render_row(f, svc_chunks[5], "BTC Explorer:", 4, app);

            // Spinner animation when a service action is in progress
            if app.service_action_busy {
                const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let frame = SPINNER[app.spinner_tick % SPINNER.len()];
                let spinner_text = Line::from(vec![
                    Span::styled(format!(" {} ", frame), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(&app.service_action_label, Style::default().fg(Color::Yellow)),
                ]);
                f.render_widget(Paragraph::new(spinner_text).alignment(ratatui::layout::Alignment::Center), svc_chunks[6]);

                let svc_help = Paragraph::new(" Please wait... ")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(svc_help, svc_chunks[7]);
            } else {
                let svc_help = Paragraph::new(" [Up/Down] Select Service  |  [Left/Right] Select Action  |  [Enter]  |  [Esc] Close ")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(svc_help, svc_chunks[7]);
            }
        }
        SettingsTab::BitcoinConf => {
            let editor_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),    // Editor area
                    Constraint::Length(1), // Help footer
                ])
                .split(chunks[1]);

            let editor_area = editor_chunks[0];
            let visible_height = editor_area.height.saturating_sub(2) as usize; // minus borders
            let visible_width = editor_area.width.saturating_sub(7) as usize; // minus borders + line numbers

            let dirty_marker = if app.conf_dirty { " * " } else { " " };
            let title = format!(" {} {}", app.conf_path, dirty_marker);

            let mut lines: Vec<Line> = Vec::new();
            let end = (app.conf_scroll + visible_height).min(app.conf_lines.len());
            for i in app.conf_scroll..end {
                let line_num = format!("{:>3} ", i + 1);
                let line_content = &app.conf_lines[i];
                let is_comment = line_content.trim_start().starts_with('#');

                if i == app.conf_cursor_y {
                    // Cursor line: build character-by-character with cursor highlight
                    let mut spans = vec![
                        Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                    ];
                    
                    let base_style = if is_comment {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let chars: Vec<char> = line_content.chars().collect();
                    // Before cursor
                    if app.conf_cursor_x > 0 {
                        let before: String = chars[..app.conf_cursor_x.min(chars.len())].iter().collect();
                        spans.push(Span::styled(before, base_style));
                    }
                    // Cursor character
                    let cursor_char = if app.conf_cursor_x < chars.len() {
                        chars[app.conf_cursor_x].to_string()
                    } else {
                        " ".to_string()
                    };
                    spans.push(Span::styled(cursor_char, Style::default().fg(Color::Black).bg(Color::Yellow)));
                    // After cursor
                    if app.conf_cursor_x + 1 < chars.len() {
                        let after: String = chars[app.conf_cursor_x + 1..].iter().collect();
                        spans.push(Span::styled(after, base_style));
                    }
                    
                    lines.push(Line::from(spans));
                } else {
                    let content_style = if is_comment {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                        Span::styled(line_content.as_str(), content_style),
                    ]));
                }
            }

            let editor = Paragraph::new(lines)
                .block(Block::default().title(title).borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
            f.render_widget(editor, editor_area);

            let status = format!(
                " Ln {}, Col {} | {} lines | [Ctrl+S] Save | [Tab] Switch Tab | [Esc] Close ",
                app.conf_cursor_y + 1,
                app.conf_cursor_x + 1,
                app.conf_lines.len(),
            );
            let help = Paragraph::new(status)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(help, editor_chunks[1]);
        }
    }
}
