use crate::tui::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let modal = centered_rect(60, 9, area);
    frame.render_widget(Clear, modal);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green))
        .title(" Add Memory ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top padding
            Constraint::Length(1), // thought field
            Constraint::Length(1), // gap
            Constraint::Length(1), // tags field
            Constraint::Length(1), // gap
            Constraint::Length(1), // hint
            Constraint::Length(1), // bottom padding
        ])
        .split(inner);

    let thought_cursor = if !app.add_focused_tags { "\u{2588}" } else { "" };
    let thought_style = if !app.add_focused_tags {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let thought_content = format!("Thought: {}{}", app.input, thought_cursor);
    frame.render_widget(
        Paragraph::new(thought_content).style(thought_style),
        chunks[1],
    );

    let tags_cursor = if app.add_focused_tags { "\u{2588}" } else { "" };
    let tags_style = if app.add_focused_tags {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let tags_content = format!("Tags:    {}{}", app.add_tags_input, tags_cursor);
    frame.render_widget(
        Paragraph::new(tags_content).style(tags_style),
        chunks[3],
    );

    let hint = Paragraph::new("[Tab switch field · Enter save · Esc cancel]")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[5]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input.clear();
            app.add_tags_input.clear();
            app.add_focused_tags = false;
            app.mode = Mode::List;
        }
        KeyCode::Tab => {
            app.add_focused_tags = !app.add_focused_tags;
        }
        KeyCode::Enter => {
            let text = app.input.trim().to_string();
            if text.is_empty() {
                return;
            }
            let tags: Vec<String> = app
                .add_tags_input
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let store = crate::store::Store::from_config();
            match store.append(&text, &tags) {
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
                    app.add_tags_input.clear();
                    app.add_focused_tags = false;
                    let _ = app.reload_memories();
                    app.mode = Mode::List;
                }
                Err(_) => {
                    // Stay in Add mode on failure (rare)
                }
            }
        }
        KeyCode::Backspace => {
            if app.add_focused_tags {
                app.add_tags_input.pop();
            } else {
                app.input.pop();
            }
        }
        KeyCode::Char(c) => {
            if app.add_focused_tags {
                app.add_tags_input.push(c);
            } else {
                app.input.push(c);
            }
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
    fn test_typing_updates_thought_input() {
        let mut app = empty_app();
        app.mode = Mode::Add;
        handle_key(&mut app, key(KeyCode::Char('h')));
        handle_key(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.input, "hi");
        assert!(app.add_tags_input.is_empty());
    }

    #[test]
    fn test_tab_switches_to_tags_field() {
        let mut app = empty_app();
        app.mode = Mode::Add;
        assert!(!app.add_focused_tags);
        handle_key(&mut app, key(KeyCode::Tab));
        assert!(app.add_focused_tags);
    }

    #[test]
    fn test_typing_in_tags_field_updates_tags_input() {
        let mut app = empty_app();
        app.mode = Mode::Add;
        app.add_focused_tags = true;
        handle_key(&mut app, key(KeyCode::Char('r')));
        handle_key(&mut app, key(KeyCode::Char('u')));
        handle_key(&mut app, key(KeyCode::Char('s')));
        handle_key(&mut app, key(KeyCode::Char('t')));
        assert_eq!(app.add_tags_input, "rust");
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_backspace_removes_from_active_field() {
        let mut app = empty_app();
        app.input = "hello".to_string();
        app.add_tags_input = "rust".to_string();

        // backspace on thought field
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.input, "hell");
        assert_eq!(app.add_tags_input, "rust");

        // switch to tags and backspace
        app.add_focused_tags = true;
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.add_tags_input, "rus");
        assert_eq!(app.input, "hell");
    }

    #[test]
    fn test_esc_clears_both_fields_and_returns_to_list() {
        let mut app = empty_app();
        app.mode = Mode::Add;
        app.input = "draft".to_string();
        app.add_tags_input = "rust".to_string();
        app.add_focused_tags = true;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
        assert!(app.input.is_empty());
        assert!(app.add_tags_input.is_empty());
        assert!(!app.add_focused_tags);
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
