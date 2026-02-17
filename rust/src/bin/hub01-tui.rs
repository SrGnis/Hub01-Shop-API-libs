//! Hub01 Shop TUI Application
//!
//! A terminal user interface for browsing Hub01 Shop projects.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

use hub01_client::{HubClient, ListProjectsParams, PaginatedResponse, Project, ProjectType};

// ============================================================================
// Application State
// ============================================================================

/// Represents the current screen being displayed
#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
    /// URL input screen - user enters API base URL
    UrlInput,
    /// Loading project types from API
    LoadingTypes,
    /// Project type selection screen
    TypeSelection,
    /// Loading projects from API
    LoadingProjects,
    /// Project table screen with pagination
    ProjectTable,
}

/// Main application state
struct AppState {
    /// Current screen being displayed
    screen: AppScreen,
    /// API base URL entered by user
    api_url: String,
    /// Cursor position in URL input
    cursor_position: usize,
    /// Initialized API client
    client: Option<HubClient>,
    /// List of available project types
    project_types: Vec<ProjectType>,
    /// Currently selected project type index
    selected_type_index: usize,
    /// Paginated response of projects
    projects: PaginatedResponse<Project>,
    /// Current page number (1-indexed)
    current_page: u32,
    /// Total number of pages
    total_pages: u32,
    /// Currently selected row in table
    selected_row: usize,
    /// Error message to display
    error_message: Option<String>,
    /// Should the application quit?
    should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: AppScreen::UrlInput,
            api_url: "https://hub01-shop.srgnis.com/api".to_string(),
            cursor_position: 36, // At end of default URL
            client: None,
            project_types: Vec::new(),
            selected_type_index: 0,
            projects: PaginatedResponse {
                data: Vec::new(),
                meta: None,
                links: None,
            },
            current_page: 1,
            total_pages: 1,
            selected_row: 0,
            error_message: None,
            should_quit: false,
        }
    }
}

impl AppState {
    /// Create a new application state with default values
    fn new() -> Self {
        Self::default()
    }

    /// Initialize the API client with the current URL
    fn init_client(&mut self) -> Result<(), String> {
        match HubClient::new(&self.api_url, None) {
            Ok(client) => {
                self.client = Some(client);
                Ok(())
            }
            Err(e) => Err(format!("Failed to create client: {}", e)),
        }
    }

    /// Fetch project types from the API
    fn fetch_project_types(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        match client.project_types().list() {
            Ok(types) => {
                self.project_types = types;
                if self.project_types.is_empty() {
                    return Err("No project types found".to_string());
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to fetch project types: {}", e)),
        }
    }

    /// Fetch projects for the selected type
    fn fetch_projects(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        let project_type = self
            .project_types
            .get(self.selected_type_index)
            .ok_or("No project type selected")?;

        let params = ListProjectsParams {
            project_type: Some(project_type.slug.clone()),
            per_page: 10,
            page: self.current_page,
            ..Default::default()
        };

        match client.projects().list(&params) {
            Ok(response) => {
                self.projects = response;
                // Calculate total pages from meta
                if let Some(meta) = &self.projects.meta {
                    if let Some(total) = meta.get("total").and_then(|t| t.as_u64()) {
                        let per_page = meta
                            .get("per_page")
                            .and_then(|p| p.as_u64())
                            .unwrap_or(10);
                        self.total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
                    }
                }
                self.selected_row = 0;
                Ok(())
            }
            Err(e) => Err(format!("Failed to fetch projects: {}", e)),
        }
    }

    /// Clear any error message
    fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set an error message
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }
}

// ============================================================================
// Terminal Setup
// ============================================================================

/// Setup the terminal for TUI mode
fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore the terminal to its original state
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

// ============================================================================
// UI Rendering
// ============================================================================

/// Render the URL input screen
fn render_url_input(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Input
            Constraint::Length(2), // Help
            Constraint::Min(1),    // Spacer
            Constraint::Length(2), // Status bar
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Hub01 Shop Browser")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Input field
    let input_block = Block::default()
        .title(" API URL ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let input_text = Paragraph::new(state.api_url.as_str())
        .style(Style::default().fg(Color::White))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Show cursor
    let cursor_x = chunks[1].x + state.cursor_position as u16 + 1;
    let cursor_y = chunks[1].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    // Help text
    let help = Paragraph::new("Press Enter to connect | Esc to quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    // Status bar
    render_status_bar(f, chunks[4], state);
}

/// Render the loading screen
fn render_loading(f: &mut Frame, state: &AppState, message: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Loading message
            Constraint::Min(1),    // Spacer
            Constraint::Length(2), // Status bar
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Hub01 Shop Browser")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Loading message
    let loading = Paragraph::new(format!("{}...", message))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(loading, chunks[1]);

    // Status bar
    render_status_bar(f, chunks[3], state);
}

/// Render the type selection screen
fn render_type_selection(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Type list
            Constraint::Length(2), // Help
            Constraint::Length(2), // Status bar
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Select Project Type")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Type list
    let rows: Vec<Row> = state
        .project_types
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == state.selected_type_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(t.icon.clone()),
                Cell::from(t.name.clone()),
                Cell::from(t.slug.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(4), Constraint::Length(20), Constraint::Length(15)],
    )
    .header(
        Row::new(vec!["", "Name", "Slug"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Project Types "),
    );
    f.render_widget(table, chunks[1]);

    // Help text
    let help = Paragraph::new("Enter: Select | Esc: Back | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    // Status bar
    render_status_bar(f, chunks[3], state);
}

/// Render the project table screen
fn render_project_table(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Project table
            Constraint::Length(2), // Help
            Constraint::Length(2), // Status bar
        ])
        .split(f.area());

    // Get selected type name
    let type_name = state
        .project_types
        .get(state.selected_type_index)
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown");

    // Title with pagination info
    let title_text = format!(
        "Projects - {} | Page {}/{}",
        type_name, state.current_page, state.total_pages
    );
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Project table
    let rows: Vec<Row> = state
        .projects
        .data
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == state.selected_row {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            // Format downloads with commas
            let downloads = format_number(p.downloads);
            
            // Truncate name if too long
            let name = if p.name.len() > 20 {
                format!("{}...", &p.name[..17])
            } else {
                p.name.clone()
            };
            
            // Format tags
            let tags = p.tags.join(", ");
            let tags = if tags.len() > 25 {
                format!("{}...", &tags[..22])
            } else {
                tags
            };

            Row::new(vec![
                Cell::from(name),
                Cell::from(downloads),
                Cell::from(p.status.clone()),
                Cell::from(tags),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(23),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Name", "Downloads", "Status", "Tags"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Projects "),
    );
    f.render_widget(table, chunks[1]);

    // Help text
    let help = Paragraph::new("n: Next | p: Prev | t: Back | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    // Status bar
    render_status_bar(f, chunks[3], state);
}

/// Render the status bar at the bottom
fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let status_text = if let Some(ref error) = state.error_message {
        format!(" Error: {}", error)
    } else {
        format!(" Connected to: {}", state.api_url)
    };

    let style = if state.error_message.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };

    let status = Paragraph::new(status_text)
        .style(style)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(status, area);
}

/// Main render function that dispatches to the appropriate screen renderer
fn render(f: &mut Frame, state: &AppState) {
    match state.screen {
        AppScreen::UrlInput => render_url_input(f, state),
        AppScreen::LoadingTypes => render_loading(f, state, "Loading project types"),
        AppScreen::TypeSelection => render_type_selection(f, state),
        AppScreen::LoadingProjects => render_loading(f, state, "Loading projects"),
        AppScreen::ProjectTable => render_project_table(f, state),
    }
}

// ============================================================================
// Event Handling
// ============================================================================

/// Handle keyboard input for URL input screen
fn handle_url_input(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Char(c) => {
            // Insert character at cursor position
            state.api_url.insert(state.cursor_position, c);
            state.cursor_position += c.len_utf8();
            state.clear_error();
        }
        KeyCode::Backspace => {
            if state.cursor_position > 0 {
                state.cursor_position -= 1;
                state.api_url.remove(state.cursor_position);
                state.clear_error();
            }
        }
        KeyCode::Delete => {
            if state.cursor_position < state.api_url.len() {
                state.api_url.remove(state.cursor_position);
                state.clear_error();
            }
        }
        KeyCode::Left => {
            if state.cursor_position > 0 {
                state.cursor_position -= 1;
            }
        }
        KeyCode::Right => {
            if state.cursor_position < state.api_url.len() {
                state.cursor_position += 1;
            }
        }
        KeyCode::Home => {
            state.cursor_position = 0;
        }
        KeyCode::End => {
            state.cursor_position = state.api_url.len();
        }
        KeyCode::Enter => {
            // Initialize client and transition to loading
            if let Err(e) = state.init_client() {
                state.set_error(e);
            } else {
                state.screen = AppScreen::LoadingTypes;
            }
        }
        KeyCode::Esc => {
            state.should_quit = true;
        }
        _ => {}
    }
}

/// Handle keyboard input for type selection screen
fn handle_type_selection(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Up => {
            if state.selected_type_index > 0 {
                state.selected_type_index -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected_type_index < state.project_types.len() - 1 {
                state.selected_type_index += 1;
            }
        }
        KeyCode::Enter => {
            state.screen = AppScreen::LoadingProjects;
            state.current_page = 1;
        }
        KeyCode::Esc => {
            state.screen = AppScreen::UrlInput;
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        _ => {}
    }
}

/// Handle keyboard input for project table screen
fn handle_project_table(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Up => {
            if state.selected_row > 0 {
                state.selected_row -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected_row < state.projects.data.len().saturating_sub(1) {
                state.selected_row += 1;
            }
        }
        KeyCode::Char('n') => {
            if state.current_page < state.total_pages {
                state.current_page += 1;
                state.screen = AppScreen::LoadingProjects;
            }
        }
        KeyCode::Char('p') => {
            if state.current_page > 1 {
                state.current_page -= 1;
                state.screen = AppScreen::LoadingProjects;
            }
        }
        KeyCode::Char('t') => {
            state.screen = AppScreen::TypeSelection;
            state.clear_error();
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        _ => {}
    }
}

/// Main event handler that dispatches to the appropriate screen handler
fn handle_event(event: Event, state: &mut AppState) {
    if let Event::Key(key_event) = event {
        // Handle 'q' globally (except in URL input where Esc is used)
        if key_event.code == KeyCode::Char('q') 
            && state.screen != AppScreen::UrlInput 
            && key_event.modifiers == KeyModifiers::NONE {
            state.should_quit = true;
            return;
        }

        match state.screen {
            AppScreen::UrlInput => handle_url_input(key_event, state),
            AppScreen::TypeSelection => handle_type_selection(key_event, state),
            AppScreen::ProjectTable => handle_project_table(key_event, state),
            _ => {} // Loading screens don't handle input
        }
    }
}

// ============================================================================
// Application Logic
// ============================================================================

/// Process the current state (e.g., fetch data during loading screens)
fn process_state(state: &mut AppState) {
    match state.screen {
        AppScreen::LoadingTypes => {
            match state.fetch_project_types() {
                Ok(()) => {
                    state.screen = AppScreen::TypeSelection;
                    state.selected_type_index = 0;
                }
                Err(e) => {
                    state.set_error(e);
                    state.screen = AppScreen::UrlInput;
                }
            }
        }
        AppScreen::LoadingProjects => {
            match state.fetch_projects() {
                Ok(()) => {
                    state.screen = AppScreen::ProjectTable;
                }
                Err(e) => {
                    state.set_error(e);
                    state.screen = AppScreen::TypeSelection;
                }
            }
        }
        _ => {}
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format a number with thousand separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

// ============================================================================
// Main Application
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create application state
    let mut state = AppState::new();

    // Main loop
    loop {
        // Draw the UI
        terminal.draw(|f| render(f, &state))?;

        // Process any state transitions (e.g., loading -> loaded)
        process_state(&mut state);

        // Check if we should quit
        if state.should_quit {
            break;
        }

        // Handle events with a timeout
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            handle_event(event, &mut state);
        }
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;

    Ok(())
}