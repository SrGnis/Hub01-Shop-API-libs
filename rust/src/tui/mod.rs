pub mod actions;
pub mod events;
pub mod state;
pub mod terminal;
pub mod ui;

pub use actions::process_state;
pub use events::handle_event;
pub use state::AppState;
pub use terminal::{restore_terminal, setup_terminal};
pub use ui::render;
