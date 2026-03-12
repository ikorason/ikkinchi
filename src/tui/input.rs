use crate::tui::app::{App, Mode};
use crate::tui::views;
use crossterm::event::KeyEvent;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode.clone() {
        Mode::List => views::list::handle_key(app, key),
        Mode::FuzzyFilter => views::list::handle_fuzzy_key(app, key),
        Mode::Add => views::add::handle_key(app, key),
        Mode::Delete => views::delete::handle_key(app, key),
        // All SemanticSearch states (Typing, Loading, Results) handled by search view
        Mode::SemanticSearch(_) => views::search::handle_key(app, key),
        Mode::View => views::thought::handle_key(app, key),
        Mode::TagFilter => {} // handled in Task 14
    }
}
