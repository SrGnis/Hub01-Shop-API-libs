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
use std::collections::HashSet;
use std::io::{self, Stdout};
use std::time::Duration;

use hub01_client::{HubClient, ListProjectsParams, PaginatedResponse, Project, ProjectTag, ProjectType, ProjectVersionTag};

// ============================================================================
// Application State
// ============================================================================

/// Represents which tag table is currently focused
#[derive(Debug, Clone, PartialEq)]
enum TagTableFocus {
    /// Project tags table is focused
    ProjectTags,
    /// Version tags table is focused
    VersionTags,
}

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
    /// Tag filter selection screen
    TagFilter,
    /// Loading tags from API
    LoadingTags,
    /// Search input screen
    SearchInput,
    /// Sort options screen
    SortOptions,
}

/// Sort field options
#[derive(Debug, Clone, PartialEq)]
enum SortField {
    Downloads,
    Name,
    UpdatedAt,
    CreatedAt,
}

impl SortField {
    fn to_api_string(&self) -> &'static str {
        match self {
            SortField::Downloads => "downloads",
            SortField::Name => "name",
            SortField::UpdatedAt => "updated_at",
            SortField::CreatedAt => "created_at",
        }
    }
}

/// Sort direction options
#[derive(Debug, Clone, PartialEq)]
enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn to_api_string(&self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }
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
    
    // Filter state
    /// List of available tags for filtering
    available_tags: Vec<ProjectTag>,
    /// Set of selected tag slugs for filtering
    selected_tags: HashSet<String>,
    /// Set of expanded main tag slugs (for hierarchy display)
    expanded_tags: HashSet<String>,
    /// Currently selected tag table
    tag_table_focus: TagTableFocus,
    /// Currently selected index within project tags table
    project_tag_selection: usize,
    /// Scroll offset for project tags table
    project_tag_scroll: usize,
    /// Currently selected index within version tags table
    version_tag_selection: usize,
    /// Scroll offset for version tags table
    version_tag_scroll: usize,
    /// Flattened list of (tag, depth) for display
    flattened_tags: Vec<(ProjectTag, usize)>,
    /// List of available version tags for filtering
    available_version_tags: Vec<ProjectVersionTag>,
    /// Set of selected version tag slugs for filtering
    selected_version_tags: HashSet<String>,
    /// Flattened list of version tags for display
    flattened_version_tags: Vec<(ProjectVersionTag, usize)>,
    
    // Search state
    /// Search query string
    search_query: String,
    /// Cursor position in search input
    search_cursor_position: usize,
    
    // Sort state
    /// Current sort field
    sort_field: SortField,
    /// Current sort direction
    sort_direction: SortDirection,
    /// Currently selected index in sort options
    sort_selection_index: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: AppScreen::UrlInput,
            api_url: "https://hub01-shop.srgnis.com/api".to_string(),
            cursor_position: "https://hub01-shop.srgnis.com/api".len(), // At end of default URL
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
            // Filter state
            available_tags: Vec::new(),
            selected_tags: HashSet::new(),
            expanded_tags: HashSet::new(),
            tag_table_focus: TagTableFocus::ProjectTags,
            project_tag_selection: 0,
            project_tag_scroll: 0,
            version_tag_selection: 0,
            version_tag_scroll: 0,
            flattened_tags: Vec::new(),
            available_version_tags: Vec::new(),
            selected_version_tags: HashSet::new(),
            flattened_version_tags: Vec::new(),
            // Search state
            search_query: String::new(),
            search_cursor_position: 0,
            // Sort state
            sort_field: SortField::Downloads,
            sort_direction: SortDirection::Desc,
            sort_selection_index: 0,
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

        // Build tags list from selected tags
        let tags: Vec<String> = self.selected_tags.iter().cloned().collect();
        
        // Build version tags list from selected version tags
        let version_tags: Vec<String> = self.selected_version_tags.iter().cloned().collect();
        
        // Build search query
        let search = if self.search_query.is_empty() {
            None
        } else {
            Some(self.search_query.clone())
        };

        let params = ListProjectsParams {
            project_type: Some(project_type.slug.clone()),
            per_page: 10,
            page: self.current_page,
            tags: if tags.is_empty() { None } else { Some(tags) },
            version_tags: if version_tags.is_empty() { None } else { Some(version_tags) },
            search,
            order_by: Some(self.sort_field.to_api_string().to_string()),
            order_direction: Some(self.sort_direction.to_api_string().to_string()),
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
    
    /// Fetch tags for the selected project type
    fn fetch_tags(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        let project_type = self
            .project_types
            .get(self.selected_type_index)
            .ok_or("No project type selected")?;

        // Fetch project tags with hierarchy
        match client.tags().list_project_tags(false, Some(&project_type.slug)) {
            Ok(tags) => {
                self.available_tags = tags;
            }
            Err(e) => return Err(format!("Failed to fetch project tags: {}", e)),
        }

        // Fetch version tags with hierarchy
        match client.tags().list_version_tags(false, Some(&project_type.slug)) {
            Ok(tags) => {
                self.available_version_tags = tags;
            }
            Err(e) => return Err(format!("Failed to fetch version tags: {}", e)),
        }

        self.flatten_tags();
        self.project_tag_selection = 0;
        self.version_tag_selection = 0;
        Ok(())
    }
    
    /// Flatten the hierarchical tags into a display list with subtags under their parents
    fn flatten_tags(&mut self) {
        self.flattened_tags.clear();
        for tag in &self.available_tags {
            self.flattened_tags.push((tag.clone(), 0));
            // Add subtags directly under their parent with indentation
            for sub_tag in &tag.sub_tags {
                self.flattened_tags.push((sub_tag.clone(), 1));
            }
        }
        
        // Also flatten version tags with hierarchy
        self.flattened_version_tags.clear();
        for tag in &self.available_version_tags {
            self.flattened_version_tags.push((tag.clone(), 0));
            for sub_tag in &tag.sub_tags {
                self.flattened_version_tags.push((sub_tag.clone(), 1));
            }
        }
    }
    
    /// Reset all filters to default state
    fn reset_filters(&mut self) {
        self.selected_tags.clear();
        self.selected_version_tags.clear();
        self.search_query.clear();
        self.search_cursor_position = 0;
        self.sort_field = SortField::Downloads;
        self.sort_direction = SortDirection::Desc;
        self.current_page = 1;
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

    // Build filter indicator string
    let mut filter_parts = Vec::new();
    if !state.selected_tags.is_empty() {
        filter_parts.push(format!("{} tags", state.selected_tags.len()));
    }
    if !state.search_query.is_empty() {
        filter_parts.push(format!("search: '{}'", state.search_query));
    }
    let filter_str = if filter_parts.is_empty() {
        String::new()
    } else {
        format!(" | Filters: {}", filter_parts.join(", "))
    };

    // Title with pagination info
    let title_text = format!(
        "Projects - {} | Page {}/{}{}",
        type_name, state.current_page, state.total_pages, filter_str
    );
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Project table with all columns
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
            
            // Truncate name if too long
            let name = if p.name.len() > 12 {
                format!("{}...", &p.name[..9])
            } else {
                p.name.clone()
            };
            
            // Truncate summary if too long
            let summary = if p.summary.len() > 20 {
                format!("{}...", &p.summary[..17])
            } else {
                p.summary.clone()
            };
            
            // Format downloads with commas
            let downloads = format_number(p.downloads);
            
            // Format updated_at date (take first 10 chars for date only)
            let updated = p.updated_at
                .as_ref()
                .map(|d| if d.len() >= 10 { &d[..10] } else { d.as_str() })
                .unwrap_or("N/A");

            // Format created_at date (take first 10 chars for date only)
            let created = if p.created_at.len() >= 10 {
                &p.created_at[..10]
            } else {
                p.created_at.as_str()
            };
            
            // Format tags
            let tags = p.tags.join(", ");
            let tags = if tags.len() > 15 {
                format!("{}...", &tags[..12])
            } else {
                tags
            };
            
            Row::new(vec![
                Cell::from(name),
                Cell::from(summary),
                Cell::from(tags),
                Cell::from(downloads),
                Cell::from(p.status.clone()),
                Cell::from(updated),
                Cell::from(created),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(23),
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["Name", "Summary", "Tags", "Downloads", "Status", "Updated", "Created"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Projects "),
    );
    f.render_widget(table, chunks[1]);

    // Help text with new keybindings
    let help = Paragraph::new("n: Next | p: Prev | f: Filter | s: Search | o: Sort | r: Reset | t: Back | q: Quit")
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
        AppScreen::TagFilter => render_tag_filter(f, state),
        AppScreen::LoadingTags => render_loading(f, state, "Loading tags"),
        AppScreen::SearchInput => render_search_input(f, state),
        AppScreen::SortOptions => render_sort_options(f, state),
    }
}

/// Ensure selection is visible by adjusting scroll offset
fn ensure_selection_visible(
    selection: usize, 
    scroll: &mut usize, 
    visible_rows: usize, 
    total_rows: usize
) {
    if visible_rows == 0 {
        return;
    }

    // If selection is above visible area
    if selection < *scroll {
        *scroll = selection;
    }
    // If selection is below visible area
    else if selection >= *scroll + visible_rows {
        *scroll = selection - visible_rows.saturating_sub(1);
    }

    // Ensure scroll doesn't exceed bounds
    *scroll = (*scroll).clamp(0, total_rows.saturating_sub(1));
}

/// Calculate number of visible rows in a table
fn calculate_visible_rows(table_height: u16) -> usize {
    // Subtract 2 for borders, 1 for top margin, 1 for bottom margin
    (table_height.saturating_sub(4)).max(0) as usize
}

/// Render the tag filter screen with side-by-side scrollable tables
fn render_tag_filter(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Min(10),    // Tables area - grows to fill space
            Constraint::Length(2),  // Help
            Constraint::Length(2),  // Status bar
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Select Tags to Filter")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Split tables area horizontally
    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Project tags (50% width)
            Constraint::Percentage(50), // Version tags (50% width)
        ])
        .split(chunks[1]);

    // Calculate visible rows for each table
    let project_visible_rows = calculate_visible_rows(table_chunks[0].height);
    let version_visible_rows = calculate_visible_rows(table_chunks[1].height);

    // Build project tag list
    let mut project_rows: Vec<Row> = Vec::new();
    let scroll = state.project_tag_scroll;
    let visible_end = scroll + project_visible_rows;
    
    for (i, (tag, depth)) in state.flattened_tags.iter().enumerate() {
        if i >= scroll && i < visible_end {
            let style = if state.tag_table_focus == TagTableFocus::ProjectTags 
                && i == state.project_tag_selection {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let indent = "    ".repeat(*depth);
            let checkbox = if state.selected_tags.contains(&tag.slug) {
                "[x]"
            } else {
                "[ ]"
            };
            
            let display_name = format!("{}{} {} {}", indent, checkbox, tag.icon, tag.name);
            
            project_rows.push(Row::new(vec![Cell::from(display_name)]).style(style));
        }
    }

    let project_table = Table::new(
        project_rows,
        [Constraint::Min(30)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Project Tags ")
            .border_style(if state.tag_table_focus == TagTableFocus::ProjectTags {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            }),
    );
    f.render_widget(project_table, table_chunks[0]);

    // Build version tag list
    let mut version_rows: Vec<Row> = Vec::new();
    let scroll = state.version_tag_scroll;
    let visible_end = scroll + version_visible_rows;
    
    for (i, (tag, depth)) in state.flattened_version_tags.iter().enumerate() {
        if i >= scroll && i < visible_end {
            let style = if state.tag_table_focus == TagTableFocus::VersionTags 
                && i == state.version_tag_selection {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let indent = "    ".repeat(*depth);
            let checkbox = if state.selected_version_tags.contains(&tag.slug) {
                "[x]"
            } else {
                "[ ]"
            };
            
            let display_name = format!("{}{} {} {}", indent, checkbox, tag.icon, tag.name);
            
            version_rows.push(Row::new(vec![Cell::from(display_name)]).style(style));
        }
    }

    let version_table = Table::new(
        version_rows,
        [Constraint::Min(30)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Version Tags ")
            .border_style(if state.tag_table_focus == TagTableFocus::VersionTags {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            }),
    );
    f.render_widget(version_table, table_chunks[1]);

    // Help text with new keybindings
    let help = Paragraph::new("Tab: Switch Table | Up/Down: Navigate | Enter: Toggle | Esc: Apply | r: Reset | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    // Status bar
    render_status_bar(f, chunks[3], state);
}

/// Render the search input screen
fn render_search_input(f: &mut Frame, state: &AppState) {
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
    let title = Paragraph::new("Search Projects")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Input field
    let input_block = Block::default()
        .title(" Search Query ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let input_text = Paragraph::new(state.search_query.as_str())
        .style(Style::default().fg(Color::White))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Show cursor
    let cursor_x = chunks[1].x + state.search_cursor_position as u16 + 1;
    let cursor_y = chunks[1].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    // Help text
    let help = Paragraph::new("Enter: Search | Esc: Cancel | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    // Status bar
    render_status_bar(f, chunks[4], state);
}

/// Render the sort options screen
fn render_sort_options(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(6), // Sort options
            Constraint::Length(2), // Help
            Constraint::Length(2), // Status bar
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Sort Options")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Sort field options
    let sort_fields = [
        ("Downloads", SortField::Downloads),
        ("Name", SortField::Name),
        ("Updated At", SortField::UpdatedAt),
        ("Created At", SortField::CreatedAt),
    ];

    let rows: Vec<Row> = sort_fields
        .iter()
        .enumerate()
        .map(|(i, (name, field))| {
            let is_selected = state.sort_field == *field;
            let is_highlighted = i == state.sort_selection_index;
            
            let style = if is_highlighted {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let marker = if is_selected { "» " } else { "  " };
            
            Row::new(vec![Cell::from(format!("{}{}", marker, name))])
                .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(20)],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Sort Field "),
    );
    f.render_widget(table, chunks[1]);
    
    // Help text
    let help = Paragraph::new("Enter: Select | d: Toggle Direction | Esc: Apply | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    // Status bar
    render_status_bar(f, chunks[3], state);
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
        KeyCode::Char('f') => {
            // Open tag filter screen
            state.screen = AppScreen::LoadingTags;
        }
        KeyCode::Char('s') => {
            // Open search input screen
            state.screen = AppScreen::SearchInput;
        }
        KeyCode::Char('o') => {
            // Open sort options screen
            state.sort_selection_index = match state.sort_field {
                SortField::Downloads => 0,
                SortField::Name => 1,
                SortField::UpdatedAt => 2,
                SortField::CreatedAt => 3,
            };
            state.screen = AppScreen::SortOptions;
        }
        KeyCode::Char('r') => {
            // Reset all filters
            state.reset_filters();
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        _ => {}
    }
}

/// Handle keyboard input for tag filter screen
fn handle_tag_filter(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Up => {
            match state.tag_table_focus {
                TagTableFocus::ProjectTags => {
                    if state.project_tag_selection > 0 {
                        state.project_tag_selection -= 1;
                        let visible_rows = 10; // Default estimate
                        ensure_selection_visible(
                            state.project_tag_selection,
                            &mut state.project_tag_scroll,
                            visible_rows,
                            state.flattened_tags.len()
                        );
                    }
                }
                TagTableFocus::VersionTags => {
                    if state.version_tag_selection > 0 {
                        state.version_tag_selection -= 1;
                        let visible_rows = 10; // Default estimate
                        ensure_selection_visible(
                            state.version_tag_selection,
                            &mut state.version_tag_scroll,
                            visible_rows,
                            state.flattened_version_tags.len()
                        );
                    }
                }
            }
        }
        KeyCode::Down => {
            match state.tag_table_focus {
                TagTableFocus::ProjectTags => {
                    if state.project_tag_selection < state.flattened_tags.len().saturating_sub(1) {
                        state.project_tag_selection += 1;
                        let visible_rows = 10; // Default estimate
                        ensure_selection_visible(
                            state.project_tag_selection,
                            &mut state.project_tag_scroll,
                            visible_rows,
                            state.flattened_tags.len()
                        );
                    }
                }
                TagTableFocus::VersionTags => {
                    if state.version_tag_selection < state.flattened_version_tags.len().saturating_sub(1) {
                        state.version_tag_selection += 1;
                        let visible_rows = 10; // Default estimate
                        ensure_selection_visible(
                            state.version_tag_selection,
                            &mut state.version_tag_scroll,
                            visible_rows,
                            state.flattened_version_tags.len()
                        );
                    }
                }
            }
        }
        KeyCode::Tab => {
            // Switch focus between tables
            state.tag_table_focus = match state.tag_table_focus {
                TagTableFocus::ProjectTags => TagTableFocus::VersionTags,
                TagTableFocus::VersionTags => TagTableFocus::ProjectTags,
            };
        }
        KeyCode::Enter => {
            // Toggle tag selection based on which table is focused
            match state.tag_table_focus {
                TagTableFocus::ProjectTags => {
                    if let Some((tag, _depth)) = state.flattened_tags.get(state.project_tag_selection) {
                        if state.selected_tags.contains(&tag.slug) {
                            state.selected_tags.remove(&tag.slug);
                        } else {
                            state.selected_tags.insert(tag.slug.clone());
                        }
                    }
                }
                TagTableFocus::VersionTags => {
                    if let Some((tag, _depth)) = state.flattened_version_tags.get(state.version_tag_selection) {
                        if state.selected_version_tags.contains(&tag.slug) {
                            state.selected_version_tags.remove(&tag.slug);
                        } else {
                            state.selected_version_tags.insert(tag.slug.clone());
                        }
                    }
                }
            }
        }
        KeyCode::Char('r') => {
            // Reset tag selection
            state.selected_tags.clear();
            state.selected_version_tags.clear();
            state.project_tag_selection = 0;
            state.version_tag_selection = 0;
            state.project_tag_scroll = 0;
            state.version_tag_scroll = 0;
            state.tag_table_focus = TagTableFocus::ProjectTags;
        }
        KeyCode::Esc => {
            // Apply and return to project table
            state.current_page = 1;
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        _ => {}
    }
}

/// Handle keyboard input for search input screen
fn handle_search_input(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Char(c) if c != 'q' => {
            // Insert character at cursor position
            state.search_query.insert(state.search_cursor_position, c);
            state.search_cursor_position += c.len_utf8();
        }
        KeyCode::Backspace => {
            if state.search_cursor_position > 0 {
                state.search_cursor_position -= 1;
                state.search_query.remove(state.search_cursor_position);
            }
        }
        KeyCode::Delete => {
            if state.search_cursor_position < state.search_query.len() {
                state.search_query.remove(state.search_cursor_position);
            }
        }
        KeyCode::Left => {
            if state.search_cursor_position > 0 {
                state.search_cursor_position -= 1;
            }
        }
        KeyCode::Right => {
            if state.search_cursor_position < state.search_query.len() {
                state.search_cursor_position += 1;
            }
        }
        KeyCode::Home => {
            state.search_cursor_position = 0;
        }
        KeyCode::End => {
            state.search_cursor_position = state.search_query.len();
        }
        KeyCode::Enter => {
            // Apply search and return to project table
            state.current_page = 1;
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Esc => {
            // Cancel and return to project table
            state.screen = AppScreen::ProjectTable;
        }
        _ => {}
    }
}

/// Handle keyboard input for sort options screen
fn handle_sort_options(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Up => {
            if state.sort_selection_index > 0 {
                state.sort_selection_index -= 1;
            }
        }
        KeyCode::Down => {
            if state.sort_selection_index < 3 {
                state.sort_selection_index += 1;
            }
        }
        KeyCode::Enter => {
            // Select the highlighted sort field
            state.sort_field = match state.sort_selection_index {
                0 => SortField::Downloads,
                1 => SortField::Name,
                2 => SortField::UpdatedAt,
                3 => SortField::CreatedAt,
                _ => SortField::Downloads,
            };
            state.current_page = 1;
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Char('d') => {
            // Toggle sort direction
            state.sort_direction = match state.sort_direction {
                SortDirection::Asc => SortDirection::Desc,
                SortDirection::Desc => SortDirection::Asc,
            };
        }
        KeyCode::Esc => {
            // Apply and return to project table
            state.current_page = 1;
            state.screen = AppScreen::LoadingProjects;
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
            AppScreen::TagFilter => handle_tag_filter(key_event, state),
            AppScreen::SearchInput => handle_search_input(key_event, state),
            AppScreen::SortOptions => handle_sort_options(key_event, state),
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
        AppScreen::LoadingTags => {
            match state.fetch_tags() {
                Ok(()) => {
                    state.screen = AppScreen::TagFilter;
                }
                Err(e) => {
                    state.set_error(e);
                    state.screen = AppScreen::ProjectTable;
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