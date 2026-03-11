use crate::tui::app::App;
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

pub fn render(_app: &App, _frame: &mut Frame, _area: Rect) {}
pub fn handle_key(_app: &mut App, _key: KeyEvent) {}
