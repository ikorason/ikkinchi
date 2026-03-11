use crate::tui::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let modal = centered_rect(60, 7, area);
    frame.render_widget(Clear, modal);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red))
        .title(" Delete Memory ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    // Split inner area: 1 line padding top, 1 line content, 1 line padding, 1 line hint
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let label = if let Some(memory) = app.selected_memory() {
        let text = &memory.text;
        let truncated = if text.chars().count() > 40 {
            let t: String = text.chars().take(40).collect();
            format!("{}\u{2026}", t)
        } else {
            text.clone()
        };
        format!("Delete \"{}\"?", truncated)
    } else {
        "Delete this memory?".to_string()
    };

    let content_widget = Paragraph::new(label)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(content_widget, chunks[1]);

    let hint = Paragraph::new("[y confirm \u{00b7} n/Esc cancel]")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[3]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = Mode::List;
        }
        KeyCode::Char('y') => {
            let memory = match app.selected_memory().cloned() {
                Some(m) => m,
                None => {
                    app.mode = Mode::List;
                    return;
                }
            };
            let _ = crate::store::Store::from_config().delete(&memory.id);
            let id_clone = memory.id.clone();
            tokio::spawn(async move {
                use crate::vectordb::VectorDb;
                if let Ok(db) = VectorDb::open().await {
                    let _ = db.delete(&id_clone).await;
                }
            });
            let _ = app.reload_memories();
            app.mode = Mode::List;
        }
        _ => {}
    }
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Memory;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn app_with_one_memory() -> App {
        App::from_memories(vec![Memory::new("2026-03-10", "14:00:00", "keep me")])
    }

    #[test]
    fn test_n_returns_to_list_unchanged() {
        let mut app = app_with_one_memory();
        app.mode = Mode::Delete;
        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.mode, Mode::List);
        assert_eq!(app.memories.len(), 1);
    }

    #[test]
    fn test_esc_returns_to_list_unchanged() {
        let mut app = app_with_one_memory();
        app.mode = Mode::Delete;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
    }
}
