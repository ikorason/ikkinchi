pub mod app;
pub mod input;
pub mod ui;
pub mod views;

use app::{App, Mode, SearchResult, SearchState};
use crate::store::Store;
use std::time::Duration;
use crossterm::event::{self, KeyEventKind};

pub async fn run() -> anyhow::Result<()> {
    let store = Store::from_config();
    let memories = store.list(usize::MAX)?;
    let mut app = App::from_memories(memories);

    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app).await;
    ratatui::restore();
    result
}

async fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(app, frame))?;

        if event::poll(Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    input::handle_key(app, key);
                }
            }
        }

        // Poll async search results
        if let Some(ref mut rx) = app.search_rx {
            use tokio::sync::mpsc::error::TryRecvError;
            match rx.try_recv() {
                Ok(SearchResult::Ok(results)) => {
                    app.visible = results;
                    app.selected = 0;
                    app.mode = Mode::SemanticSearch(SearchState::Results);
                    app.search_rx = None;
                }
                Ok(SearchResult::Err(msg)) => {
                    app.error = Some(msg);
                    app.search_rx = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    app.search_rx = None;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
