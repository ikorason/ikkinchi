use crate::store::Memory;
use crate::tui::app::{App, Mode, SearchResult, SearchState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    match &app.mode {
        Mode::SemanticSearch(SearchState::Results) => {
            // No modal — list view handles rendering
            return;
        }
        Mode::SemanticSearch(SearchState::Typing) | Mode::SemanticSearch(SearchState::Loading) => {
            // fall through to render modal
        }
        _ => return,
    }

    let modal = centered_rect(60, 7, area);
    frame.render_widget(Clear, modal);

    // Choose border color based on state
    let border_color = if matches!(&app.mode, Mode::SemanticSearch(SearchState::Loading)) {
        Color::Yellow
    } else {
        Color::Blue
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(" Semantic Search ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    // Split inner area into 5 lines: line0, line1, line2, line3, line4
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    if let Some(err) = &app.error {
        // Error state: show error on line 0, hint on line 2
        let error_widget = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error_widget, chunks[0]);

        let hint = Paragraph::new("[Esc to dismiss]")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[2]);
    } else if matches!(&app.mode, Mode::SemanticSearch(SearchState::Loading)) {
        // Loading state: "Searching..." centered on line 1, yellow
        let loading = Paragraph::new("Searching...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(loading, chunks[1]);
    } else {
        // Typing state
        let content = format!("Query: {}\u{2588}", app.input);
        let content_widget = Paragraph::new(content)
            .style(Style::default().fg(Color::White));
        frame.render_widget(content_widget, chunks[0]);

        let hint = Paragraph::new("[Enter to search \u{00b7} Esc cancel]")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[2]);
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    use KeyCode::{Backspace, Char, Down, Enter, Esc, Up};
    use SearchState::{Loading, Results, Typing};

    match (&app.mode.clone(), key.code) {
        // Loading: only Esc works
        (Mode::SemanticSearch(Loading), Esc) => {
            app.search_rx = None;
            app.error = None;
            app.reset_to_full_list();
            app.mode = Mode::List;
        }
        (Mode::SemanticSearch(Loading), _) => {} // ignore all other keys

        // Error shown in Typing state: Esc dismisses
        (Mode::SemanticSearch(Typing), Esc) if app.error.is_some() => {
            app.error = None;
            app.reset_to_full_list();
            app.mode = Mode::List;
        }

        // Normal Typing
        (Mode::SemanticSearch(Typing), Esc) => {
            app.input.clear();
            app.reset_to_full_list();
            app.mode = Mode::List;
        }
        (Mode::SemanticSearch(Typing), Enter) => {
            let query = app.input.trim().to_string();
            if query.is_empty() {
                return;
            }
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            app.search_rx = Some(rx);
            app.mode = Mode::SemanticSearch(SearchState::Loading);
            let memories_snapshot = app.memories.clone();
            tokio::spawn(async move {
                let result = run_semantic_search(&query, &memories_snapshot).await;
                let _ = tx.send(result).await;
            });
        }
        (Mode::SemanticSearch(Typing), Backspace) => {
            app.input.pop();
        }
        (Mode::SemanticSearch(Typing), Char(c)) => {
            app.input.push(c);
        }

        // Results: j/k navigate, Esc restores full list
        (Mode::SemanticSearch(Results), Esc) => {
            app.reset_to_full_list();
            app.mode = Mode::List;
        }
        (Mode::SemanticSearch(Results), Char('j')) | (Mode::SemanticSearch(Results), Down) => {
            app.select_next()
        }
        (Mode::SemanticSearch(Results), Char('k')) | (Mode::SemanticSearch(Results), Up) => {
            app.select_prev()
        }
        _ => {}
    }
}

async fn run_semantic_search(query: &str, memories: &[Memory]) -> SearchResult {
    use crate::config::Config;
    use crate::embed::EmbedClient;
    use crate::vectordb::VectorDb;
    use rig::embeddings::Embedding;
    use rig::embeddings::distance::VectorDistance;

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => return SearchResult::Err(format!("Config error: {}", e)),
    };
    let embed_client = match EmbedClient::from_config(&config) {
        Ok(c) => c,
        Err(_) => {
            return SearchResult::Err("Ollama unavailable. Start with: ollama serve".into())
        }
    };
    let vector_db = match VectorDb::open().await {
        Ok(db) => db,
        Err(_) => return SearchResult::Err("No vectors found. Run: ikkinchi reindex".into()),
    };
    let rows = match vector_db.load_all().await {
        Ok(r) => r,
        Err(e) => return SearchResult::Err(format!("Vector DB error: {}", e)),
    };
    if rows.is_empty() {
        return SearchResult::Err("No vectors found. Run: ikkinchi reindex".into());
    }
    let query_vec = match embed_client.embed_query(query).await {
        Ok(v) => Embedding {
            document: String::new(),
            vec: v,
        },
        Err(_) => {
            return SearchResult::Err("Ollama unavailable. Start with: ollama serve".into())
        }
    };
    let limit = config.display.list_count;
    let mut scored: Vec<(String, f64)> = rows
        .into_iter()
        .map(|(id, vec)| {
            let stored = Embedding {
                document: String::new(),
                vec,
            };
            let score = query_vec.cosine_similarity(&stored, false);
            (id, score)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    let memory_map: std::collections::HashMap<&str, &Memory> =
        memories.iter().map(|m| (m.id.as_str(), m)).collect();
    let results: Vec<Memory> = scored
        .iter()
        .filter_map(|(id, _)| memory_map.get(id.as_str()).copied().cloned())
        .collect();
    SearchResult::Ok(results)
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
    fn test_typing_updates_input() {
        let mut app = two_item_app();
        app.mode = Mode::SemanticSearch(SearchState::Typing);
        handle_key(&mut app, key(KeyCode::Char('r')));
        handle_key(&mut app, key(KeyCode::Char('u')));
        handle_key(&mut app, key(KeyCode::Char('s')));
        handle_key(&mut app, key(KeyCode::Char('t')));
        assert_eq!(app.input, "rust");
    }

    #[test]
    fn test_backspace_removes_char() {
        let mut app = two_item_app();
        app.mode = Mode::SemanticSearch(SearchState::Typing);
        app.input = "rust".to_string();
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.input, "rus");
    }

    #[test]
    fn test_esc_typing_returns_to_list() {
        let mut app = two_item_app();
        app.mode = Mode::SemanticSearch(SearchState::Typing);
        app.input = "query".to_string();
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_esc_results_restores_full_list() {
        let mut app = two_item_app();
        app.mode = Mode::SemanticSearch(SearchState::Results);
        app.input = "query".to_string();
        app.visible = vec![make_memory("2026-03-10/14:00:00", "first memory")];
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::List);
        assert_eq!(app.visible.len(), 2);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_enter_on_empty_query_does_nothing() {
        let mut app = two_item_app();
        app.mode = Mode::SemanticSearch(SearchState::Typing);
        app.input = String::new();
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(matches!(app.mode, Mode::SemanticSearch(SearchState::Typing)));
    }

    #[tokio::test]
    async fn test_enter_with_query_transitions_to_loading() {
        let mut app = two_item_app();
        app.mode = Mode::SemanticSearch(SearchState::Typing);
        app.input = "something".to_string();
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(matches!(
            app.mode,
            Mode::SemanticSearch(SearchState::Loading)
        ));
    }
}
