use crate::tui::app::{App, Mode, SearchState};
use crate::tui::views;
use crossterm::event::KeyEvent;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode.clone() {
        Mode::List => views::list::handle_key(app, key),
        Mode::FuzzyFilter => views::list::handle_fuzzy_key(app, key),
        Mode::Add => views::add::handle_key(app, key),
        Mode::Delete => views::delete::handle_key(app, key),
        // Results: routed to search view (not list) to prevent 's' re-opening search
        Mode::SemanticSearch(SearchState::Results) => views::search::handle_key(app, key),
        Mode::SemanticSearch(_) => views::search::handle_key(app, key),
    }
}
