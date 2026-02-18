//! Hub01 Shop TUI Application
//!
//! A terminal user interface for browsing Hub01 Shop projects.

use crossterm::event;
use crate::tui::{
    handle_event, process_state, render, restore_terminal, setup_terminal, AppState,
};
use std::time::Duration;

mod tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = setup_terminal()?;
    let mut state = AppState::new();

    loop {
        terminal.draw(|f| render(f, &state))?;

        process_state(&mut state);

        if state.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            handle_event(event, &mut state);
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
