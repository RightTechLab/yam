use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn render_playground(f: &mut Frame, app: &App) {
    let area = f.area();

    // Pre-calculate chunks based on if we have suggestions to show
    let has_suggestions = !app.playground_suggestions.is_empty();
    let suggestion_height = if has_suggestions {
        app.playground_suggestions.len().min(5) as u16 + 2 // Max 5 items + borders
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5), 
            Constraint::Length(suggestion_height), // Auto-collapse to 0 if none
            Constraint::Length(3)
        ])
        .split(area);

    f.render_widget(Clear, chunks[0]);
    if has_suggestions { f.render_widget(Clear, chunks[1]); }
    f.render_widget(Clear, chunks[2]);

    // --- Output History ---
    let mut history_lines = Vec::new();
    for row in &app.playground_history {
        // We split on \n in case a command returns multiline stdout
        for subline in row.lines() {
            // Apply a simple logic to colorize user input vs shell output
            if subline.starts_with("> ") {
                history_lines.push(Line::from(Span::styled(
                    subline,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
            } else if subline.starts_with("Error:") {
                history_lines.push(Line::from(Span::styled(
                    subline,
                    Style::default().fg(Color::Red),
                )));
            } else {
                history_lines.push(Line::from(Span::styled(
                    subline,
                    Style::default().fg(Color::White),
                )));
            }
        }
    }

    // Calculate total lines, we might have scrolled up
    let total_lines = history_lines.len();
    
    // Calculate the amount of printable lines inside the border block
    let render_height = chunks[0].height.saturating_sub(2) as usize; // Account for borders
    
    // apply scroll offset
    // scroll=0 means bottom. scroll=N means N lines scrolled up.
    let max_scroll = total_lines.saturating_sub(render_height);
    let effective_scroll = app.playground_scroll.min(max_scroll);
    
    let start_idx = total_lines.saturating_sub(render_height + effective_scroll);
    let viewable_lines = history_lines.into_iter().skip(start_idx).take(render_height).collect::<Vec<_>>();

    let output_block = Paragraph::new(viewable_lines)
        .block(Block::default().title(" Bitcoin CLI Playground ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(output_block, chunks[0]);

    // --- Autocomplete Box ---
    if has_suggestions {
        let items: Vec<ListItem> = app.playground_suggestions.iter().enumerate().map(|(i, s)| {
            let mut style = Style::default().fg(Color::Cyan);
            if Some(i) == app.playground_suggestion_idx {
                style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
            }
            ListItem::new(Span::styled(s, style))
        }).collect();

        let list_block = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Suggestions "));
        f.render_widget(list_block, chunks[1]);
    }

    // --- Input Box ---
    // The cursor logic is a bit crude here but works for a single line text input
    let input_text = format!("> {}", app.playground_input);
    let input_block = Paragraph::new(input_text.as_str())
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input ([Enter] Execute | [Tab] Autocomplete | 'clear' to reset | [Shift+Up/Dn] Scroll | [Esc] Close) "),
        );
    f.render_widget(input_block, chunks[2]);
}
