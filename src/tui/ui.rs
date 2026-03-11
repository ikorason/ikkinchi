use crate::tui::app::{App, Mode, SearchState};
use crate::tui::views;
use ratatui::Frame;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    match &app.mode {
        Mode::List | Mode::FuzzyFilter => views::list::render(app, frame, area),
        Mode::Add => {
            views::list::render(app, frame, area);
            views::add::render(app, frame, area);
        }
        Mode::Delete => {
            views::list::render(app, frame, area);
            views::delete::render(app, frame, area);
        }
        Mode::SemanticSearch(SearchState::Results) => views::list::render(app, frame, area),
        Mode::SemanticSearch(_) => {
            views::list::render(app, frame, area);
            views::search::render(app, frame, area);
        }
    }
}
