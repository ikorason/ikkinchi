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
        .border_style(Style::default().fg(Color::Green))
        .title(" Add Memory ")
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

    let content = format!("Thought: {}\u{2588}", app.input);
    let content_widget = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    frame.render_widget(content_widget, chunks[1]);

    let hint = Paragraph::new("[Enter to save · Esc cancel]")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[3]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input.clear();
            app.mode = Mode::List;
        }
        KeyCode::Enter => {
            let text = app.input.trim().to_string();
            if text.is_empty() {
                return;
            }
            let store = crate::store::Store::from_config();
            match store.append(&text, &[]) {
                Ok(id) => {
                    let text_clone = text.clone();
                    let id_clone = id.clone();
                    tokio::spawn(async move {
                        use crate::config::Config;
                        use crate::embed::EmbedClient;
                        use crate::vectordb::VectorDb;
                        if let Ok(config) = Config::load()
                            && let Ok(client) = EmbedClient::from_config(&config)
                            && let Ok(vec) = client.embed_document(&text_clone).await
                            && let Ok(db) = VectorDb::open().await
                        {
                            let _ = db.insert(&id_clone, &vec).await;
                        }
                    });
                    app.input.clear();
                    let _ = app.reload_memories();
                    app.mode = Mode::List;
                }
                Err(_) => {
                    // Stay in Add mode on failure (rare)
                }
            }
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(c) => {
            app.input.push(c);
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
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn empty_app() -> App {
        App::from_memories(vec![])
    }

    #[test]
    fn test_typing_updates_input() {
        let mut app = empty_app();
        app.mode = Mode::Add;
        handle_key(&mut app, key(KeyCode::Char('h')));
        handle_key(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.input, "hi");
    }

    #[test]
    fn test_backspace_removes_char() {
        let mut app = empty_app();
        app.input = "hello".to_string();
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.input, "hell");
    }

    #[test]
    fn test_esc_clears_input_and_returns_to_list() {
        let mut app = empty_app();
        app.mode = Mode::Add;
        app.input = "draft".to_string();
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_enter_on_empty_input_does_nothing() {
        let mut app = empty_app();
        app.mode = Mode::Add;
        app.input = String::new();
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Add);
    }
}
