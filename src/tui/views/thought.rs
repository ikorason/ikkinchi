use crate::tui::app::{App, Mode};
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

    let text_widget = Paragraph::new(memory.text.as_str())
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
    frame.render_widget(text_widget, inner);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::List;
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
    fn test_esc_returns_to_list() {
        let mut app = app_with_memory();
        app.mode = Mode::View;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
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
