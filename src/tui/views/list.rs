use crate::tui::app::{App, Mode, SearchState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let is_filtered = matches!(
        &app.mode,
        Mode::FuzzyFilter | Mode::SemanticSearch(SearchState::Results)
    );

    let (border_color, border_type, title_right) = if is_filtered {
        let query = if app.input.chars().count() > 20 {
            let truncated: String = app.input.chars().take(20).collect();
            format!("{}…", truncated)
        } else {
            app.input.clone()
        };
        (
            Color::Cyan,
            BorderType::Double,
            format!(" Search: \"{}\" ", query),
        )
    } else {
        (Color::Gray, BorderType::Plain, " [List] ".to_string())
    };

    let footer_text = if is_filtered {
        let count = app.visible.len();
        format!(" Esc to return · {} results ", count)
    } else {
        " / fuzzy · s search · a add · d del · q ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(title_right).alignment(Alignment::Right))
        .title_bottom(Line::from(footer_text).alignment(Alignment::Left));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // In FuzzyFilter mode: allocate 1 line for input bar at top
    let (input_area, list_area) = if matches!(&app.mode, Mode::FuzzyFilter) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, inner)
    };

    // Render fuzzy input bar
    if let Some(ia) = input_area {
        let input_text = format!("/ {}", app.input);
        let input_widget = Paragraph::new(input_text)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(input_widget, ia);
    }

    // Determine what to show in empty state
    if app.visible.is_empty() {
        let empty_msg = if app.memories.is_empty() {
            "No memories yet. Press 'a' to add your first thought."
        } else {
            "No matches."
        };
        let empty_widget = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(empty_widget, list_area);
        return;
    }

    // Build list items
    let items: Vec<ListItem> = app
        .visible
        .iter()
        .enumerate()
        .map(|(i, memory)| {
            let text_display = if memory.text.chars().count() > 50 {
                let truncated: String = memory.text.chars().take(50).collect();
                format!("{}…", truncated)
            } else {
                memory.text.clone()
            };

            if i == app.selected {
                let line = format!("> {:>3}  {}  {}", i + 1, memory.id, text_display);
                ListItem::new(line)
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                let line = format!("  {:>3}  {}  {}", i + 1, memory.id, text_display);
                ListItem::new(line)
            }
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.selected));
    let list_widget = List::new(items);
    frame.render_stateful_widget(list_widget, list_area, &mut list_state);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('a') => {
            app.input.clear();
            app.mode = Mode::Add;
        }
        KeyCode::Char('d') => {
            if app.selected_memory().is_some() {
                app.mode = Mode::Delete;
            }
        }
        KeyCode::Char('/') => {
            app.input.clear();
            app.mode = Mode::FuzzyFilter;
        }
        KeyCode::Char('s') => {
            app.input.clear();
            app.mode = Mode::SemanticSearch(SearchState::Typing);
        }
        _ => {}
    }
}

pub fn handle_fuzzy_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.reset_to_full_list();
            app.mode = Mode::List;
        }
        KeyCode::Char(c) => {
            app.input.push(c);
            apply_fuzzy_filter(app);
        }
        KeyCode::Backspace => {
            app.input.pop();
            apply_fuzzy_filter(app);
        }
        KeyCode::Down => app.select_next(),
        KeyCode::Up => app.select_prev(),
        _ => {}
    }
}

fn apply_fuzzy_filter(app: &mut App) {
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    if app.input.is_empty() {
        app.visible = app.memories.clone();
        app.selected = 0;
        return;
    }

    let matcher = SkimMatcherV2::default();
    let query = app.input.clone();
    let mut scored: Vec<(i64, usize)> = app
        .memories
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            matcher.fuzzy_match(&m.text, &query).map(|score| (score, i))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    app.visible = scored.into_iter().map(|(_, i)| app.memories[i].clone()).collect();
    app.selected = 0;
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

    fn make_memory(id: &str, text: &str) -> Memory {
        Memory::new(
            id.split('/').next().unwrap(),
            id.split('/').nth(1).unwrap(),
            text,
        )
    }

    fn two_item_app() -> App {
        App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "first memory"),
            make_memory("2026-03-10/15:00:00", "second memory"),
        ])
    }

    #[test]
    fn test_j_moves_down() {
        let mut app = two_item_app();
        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_k_moves_up() {
        let mut app = two_item_app();
        app.selected = 1;
        handle_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_down_arrow_moves_down() {
        let mut app = two_item_app();
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_up_arrow_moves_up() {
        let mut app = two_item_app();
        app.selected = 1;
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_q_sets_should_quit() {
        let mut app = two_item_app();
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_a_enters_add_mode() {
        let mut app = two_item_app();
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.mode, Mode::Add);
    }

    #[test]
    fn test_d_enters_delete_mode() {
        let mut app = two_item_app();
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.mode, Mode::Delete);
    }

    #[test]
    fn test_slash_enters_fuzzy_filter_mode() {
        let mut app = two_item_app();
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert_eq!(app.mode, Mode::FuzzyFilter);
    }

    #[test]
    fn test_s_enters_semantic_search_mode() {
        let mut app = two_item_app();
        handle_key(&mut app, key(KeyCode::Char('s')));
        assert!(matches!(app.mode, Mode::SemanticSearch(_)));
    }

    #[test]
    fn test_d_on_empty_list_does_nothing() {
        let mut app = App::from_memories(vec![]);
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.mode, Mode::List);
    }

    // FuzzyFilter tests

    #[test]
    fn test_fuzzy_typing_updates_input() {
        let mut app = two_item_app();
        app.mode = Mode::FuzzyFilter;
        handle_fuzzy_key(&mut app, key(KeyCode::Char('f')));
        assert_eq!(app.input, "f");
    }

    #[test]
    fn test_fuzzy_typing_filters_visible() {
        let mut app = App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "foo bar"),
            make_memory("2026-03-10/15:00:00", "something else"),
        ]);
        app.mode = Mode::FuzzyFilter;
        handle_fuzzy_key(&mut app, key(KeyCode::Char('f')));
        handle_fuzzy_key(&mut app, key(KeyCode::Char('o')));
        handle_fuzzy_key(&mut app, key(KeyCode::Char('o')));
        assert_eq!(app.visible.len(), 1);
        assert!(app.visible[0].text.contains("foo"));
    }

    #[test]
    fn test_fuzzy_empty_query_shows_full_list() {
        let mut app = App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "foo bar"),
            make_memory("2026-03-10/15:00:00", "something else"),
        ]);
        app.mode = Mode::FuzzyFilter;
        app.input = "xyz".to_string();
        app.visible = vec![];
        handle_fuzzy_key(&mut app, key(KeyCode::Backspace));
        handle_fuzzy_key(&mut app, key(KeyCode::Backspace));
        handle_fuzzy_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.visible.len(), 2);
    }

    #[test]
    fn test_fuzzy_esc_returns_to_list() {
        let mut app = App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "foo bar"),
            make_memory("2026-03-10/15:00:00", "something else"),
        ]);
        app.mode = Mode::FuzzyFilter;
        app.input = "foo".to_string();
        handle_fuzzy_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
        assert!(app.input.is_empty());
        assert_eq!(app.visible.len(), 2);
    }

    #[test]
    fn test_fuzzy_j_k_navigate_results() {
        let mut app = two_item_app();
        app.mode = Mode::FuzzyFilter;
        handle_fuzzy_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected, 1);
        handle_fuzzy_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected, 0);
    }
}
