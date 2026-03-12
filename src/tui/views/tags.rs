use crate::store::Memory;
use crate::tui::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

/// Build a sorted list of (tag, count) from memories.
pub fn all_tags_sorted(memories: &[Memory]) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for m in memories {
        for t in &m.tags {
            *counts.entry(t.as_str()).or_insert(0) += 1;
        }
    }
    let mut sorted: Vec<(String, usize)> = counts
        .into_iter()
        .map(|(t, c)| (t.to_string(), c))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    sorted
}

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Style},
        text::Line,
        widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
    };

    let tags = all_tags_sorted(&app.memories);

    let height = (tags.len() as u16 + 4).min(20).max(5);
    let modal_area = {
        let vert = Layout::default()
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
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(vert[1])[1]
    };

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Line::from(" Filter by tag ").alignment(Alignment::Center))
        .title_bottom(Line::from(" ↵ select · Esc cancel ").alignment(Alignment::Center));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    if tags.is_empty() {
        let empty = ratatui::widgets::Paragraph::new("No tags yet.")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = tags
        .iter()
        .enumerate()
        .map(|(i, (tag, count))| {
            let line = format!("  #{:<20}  {}", tag, count);
            if i == app.tag_picker_selected {
                ListItem::new(line).style(Style::default().fg(Color::Yellow))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(app.tag_picker_selected));
    frame.render_stateful_widget(List::new(items), inner, &mut state);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let tags = all_tags_sorted(&app.memories);
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::List;
            app.tag_picker_selected = 0;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !tags.is_empty() && app.tag_picker_selected < tags.len() - 1 {
                app.tag_picker_selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.tag_picker_selected > 0 {
                app.tag_picker_selected -= 1;
            }
        }
        KeyCode::Enter => {
            if let Some((tag, _)) = tags.get(app.tag_picker_selected) {
                let t = tag.clone();
                app.active_tag_filter = Some(t.clone());
                app.visible = app.memories.iter().filter(|m| m.tags.contains(&t)).cloned().collect();
                app.selected = 0;
                app.mode = Mode::List;
                app.tag_picker_selected = 0;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Memory;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }
    }

    fn make_memory(tags: Vec<&str>) -> Memory {
        let mut m = Memory::new("2026-03-11", "10:00:00", "text");
        m.tags = tags.into_iter().map(|s| s.to_string()).collect();
        m
    }

    #[test]
    fn test_all_tags_sorted_empty() {
        assert!(all_tags_sorted(&[]).is_empty());
    }

    #[test]
    fn test_all_tags_sorted_by_count_descending() {
        let memories = vec![
            make_memory(vec!["rust", "til"]),
            make_memory(vec!["rust"]),
        ];
        let result = all_tags_sorted(&memories);
        assert_eq!(result[0].0, "rust");
        assert_eq!(result[0].1, 2);
        assert_eq!(result[1].0, "til");
    }

    #[test]
    fn test_handle_key_esc_returns_to_list() {
        let mut app = App::from_memories(vec![make_memory(vec!["rust"])]);
        app.mode = Mode::TagFilter;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
        assert_eq!(app.tag_picker_selected, 0);
        assert!(app.active_tag_filter.is_none()); // Esc does NOT set filter
    }

    #[test]
    fn test_handle_key_j_moves_cursor_down() {
        let memories = vec![
            make_memory(vec!["rust"]),
            make_memory(vec!["til"]),
        ];
        let mut app = App::from_memories(memories);
        app.mode = Mode::TagFilter;
        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.tag_picker_selected, 1);
    }

    #[test]
    fn test_handle_key_k_moves_cursor_up() {
        let memories = vec![
            make_memory(vec!["rust"]),
            make_memory(vec!["til"]),
        ];
        let mut app = App::from_memories(memories);
        app.mode = Mode::TagFilter;
        app.tag_picker_selected = 1;
        handle_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.tag_picker_selected, 0);
    }

    #[test]
    fn test_handle_key_enter_sets_filter_and_filters_visible() {
        let memories = vec![
            make_memory(vec!["rust"]),
            make_memory(vec!["til"]),
        ];
        let mut app = App::from_memories(memories);
        app.mode = Mode::TagFilter;
        app.tag_picker_selected = 0;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.mode, Mode::List);
        assert!(app.active_tag_filter.is_some());
        assert_eq!(app.visible.len(), 1);
    }
}
