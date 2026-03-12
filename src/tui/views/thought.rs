use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let modal = centered_rect(70, 14, area);
    frame.render_widget(Clear, modal);

    let memory = match app.selected_memory() {
        Some(m) => m,
        None => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Line::from(format!(" {} ", memory.id)).alignment(Alignment::Center))
        .title_bottom(Line::from(" [Esc to close · q to quit] ").alignment(Alignment::Center));

    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    if memory.tags.is_empty() {
        let text_widget = Paragraph::new(memory.text.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(text_widget, inner);
    } else {
        let tags_display = memory.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ");
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);
        let tags_widget = Paragraph::new(format!("Tags: {}", tags_display))
            .style(Style::default().fg(Color::Gray));
        let text_widget = Paragraph::new(memory.text.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(tags_widget, chunks[0]);
        frame.render_widget(text_widget, chunks[1]);
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = app.prev_mode.clone();
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
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
    use crate::tui::app::Mode;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn app_with_memory() -> App {
        App::from_memories(vec![Memory::new(
            "2026-03-11",
            "10:00:00",
            "A long thought that should be displayed in full without truncation.",
        )])
    }

    #[test]
    fn test_esc_returns_to_prev_mode() {
        let mut app = app_with_memory();
        app.prev_mode = Mode::List;
        app.mode = Mode::View;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
    }

    #[test]
    fn test_esc_returns_to_search_results() {
        use crate::tui::app::SearchState;
        let mut app = app_with_memory();
        app.prev_mode = Mode::SemanticSearch(SearchState::Results);
        app.mode = Mode::View;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::SemanticSearch(SearchState::Results));
    }

    #[test]
    fn test_q_sets_should_quit() {
        let mut app = app_with_memory();
        app.mode = Mode::View;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(app.should_quit);
        assert_eq!(app.mode, Mode::View); // mode unchanged, just quit flag set
    }

    #[test]
    fn test_other_keys_ignored() {
        let mut app = app_with_memory();
        app.mode = Mode::View;
        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.mode, Mode::View);
        assert!(!app.should_quit);
    }
}
