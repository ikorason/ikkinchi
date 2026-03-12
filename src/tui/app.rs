use crate::store::{Memory, Store};
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum SearchResult {
    Ok(Vec<Memory>),
    Err(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchState {
    Typing,
    Loading,
    Results,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    List,
    FuzzyFilter,
    Add,
    Delete,
    SemanticSearch(SearchState),
    View,
    TagFilter,
}

pub struct App {
    pub mode: Mode,
    pub prev_mode: Mode,
    pub memories: Vec<Memory>,
    pub visible: Vec<Memory>,
    pub selected: usize,
    pub input: String,
    pub search_rx: Option<Receiver<SearchResult>>,
    pub error: Option<String>,
    pub should_quit: bool,
    pub active_tag_filter: Option<String>,
    pub tag_picker_selected: usize,
    pub add_tags_input: String,
    pub add_focused_tags: bool,
}

impl App {
    pub fn from_memories(memories: Vec<Memory>) -> Self {
        let visible = memories.clone();
        Self {
            mode: Mode::List,
            prev_mode: Mode::List,
            memories,
            visible,
            selected: 0,
            input: String::new(),
            search_rx: None,
            error: None,
            should_quit: false,
            active_tag_filter: None,
            tag_picker_selected: 0,
            add_tags_input: String::new(),
            add_focused_tags: false,
        }
    }

    pub fn select_next(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        let max = self.visible.len() - 1;
        if self.selected < max {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn reset_to_full_list(&mut self) {
        self.visible = self.memories.clone();
        self.selected = 0;
        self.input.clear();
    }

    pub fn reload_memories(&mut self) -> anyhow::Result<()> {
        let memories = Store::from_config().list(usize::MAX)?;
        self.memories = memories.clone();
        self.visible = memories;
        self.selected = 0;
        self.input.clear();
        Ok(())
    }

    pub fn selected_memory(&self) -> Option<&Memory> {
        self.visible.get(self.selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_memory(id: &str, text: &str) -> Memory {
        Memory::new(
            id.split('/').next().unwrap(),
            id.split('/').nth(1).unwrap(),
            text,
        )
    }

    #[test]
    fn test_app_new_starts_in_list_mode() {
        let app = App::from_memories(vec![make_memory("2026-03-10/14:00:00", "first")]);
        assert_eq!(app.mode, Mode::List);
        assert_eq!(app.selected, 0);
        assert_eq!(app.visible.len(), 1);
        assert_eq!(app.memories.len(), 1);
    }

    #[test]
    fn test_app_select_next_increments() {
        let mut app = App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "first"),
            make_memory("2026-03-10/15:00:00", "second"),
        ]);
        app.select_next();
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_app_select_next_clamps_at_bottom() {
        let mut app = App::from_memories(vec![make_memory("2026-03-10/14:00:00", "only")]);
        app.select_next();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_app_select_prev_decrements() {
        let mut app = App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "first"),
            make_memory("2026-03-10/15:00:00", "second"),
        ]);
        app.selected = 1;
        app.select_prev();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_app_select_prev_clamps_at_top() {
        let mut app = App::from_memories(vec![make_memory("2026-03-10/14:00:00", "only")]);
        app.select_prev();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_app_reset_to_full_list() {
        let mut app = App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "first"),
            make_memory("2026-03-10/15:00:00", "second"),
        ]);
        app.visible = vec![make_memory("2026-03-10/14:00:00", "first")];
        app.selected = 5;
        app.input = "some query".to_string();
        app.reset_to_full_list();
        assert_eq!(app.visible.len(), 2);
        assert_eq!(app.selected, 0);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_app_selected_memory_returns_current() {
        let mut app = App::from_memories(vec![
            make_memory("2026-03-10/14:00:00", "first"),
            make_memory("2026-03-10/15:00:00", "second"),
        ]);
        app.selected = 1;
        assert_eq!(app.selected_memory().unwrap().text, "second");
    }

    #[test]
    fn test_app_selected_memory_returns_none_when_empty() {
        let app = App::from_memories(vec![]);
        assert!(app.selected_memory().is_none());
    }

    #[test]
    fn test_app_has_no_active_tag_filter_by_default() {
        let app = App::from_memories(vec![make_memory("2026-03-10/14:00:00", "first")]);
        assert!(app.active_tag_filter.is_none());
        assert_eq!(app.tag_picker_selected, 0);
    }

    #[test]
    fn test_tag_filter_mode_exists() {
        let mut app = App::from_memories(vec![make_memory("2026-03-10/14:00:00", "first")]);
        app.mode = Mode::TagFilter;
        assert_eq!(app.mode, Mode::TagFilter);
    }
}
