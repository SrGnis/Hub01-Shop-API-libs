use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use super::state::{AppScreen, AppState, SortField, TagTableFocus};

/// Main render function that dispatches to the appropriate screen renderer.
pub fn render(f: &mut Frame, state: &AppState) {
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

/// Render the URL input screen.
fn render_url_input(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(f.area());

    let title = Paragraph::new("Hub01 Shop Browser")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .title(" API URL ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_text = Paragraph::new(state.api_url.as_str())
        .style(Style::default().fg(Color::White))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    let cursor_x = chunks[1].x + state.cursor_position as u16 + 1;
    let cursor_y = chunks[1].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    let help = Paragraph::new("Press Enter to connect | Esc to quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    render_status_bar(f, chunks[4], state);
}

/// Render the loading screen.
fn render_loading(f: &mut Frame, state: &AppState, message: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(f.area());

    let title = Paragraph::new("Hub01 Shop Browser")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let loading = Paragraph::new(format!("{}...", message))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(loading, chunks[1]);

    render_status_bar(f, chunks[3], state);
}

/// Render the type selection screen.
fn render_type_selection(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(f.area());

    let title = Paragraph::new("Select Project Type")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

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
            Row::new(vec![Cell::from(t.name.clone()), Cell::from(t.slug.clone())]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(15),
        ],
    )
    .header(
        Row::new(vec!["Name", "Slug"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Project Types "),
    );
    f.render_widget(table, chunks[1]);

    let help = Paragraph::new("Enter: Select | Esc: Back | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    render_status_bar(f, chunks[3], state);
}

/// Render the project table screen.
fn render_project_table(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(f.area());

    let type_name = state
        .project_types
        .get(state.selected_type_index)
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown");

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

    let title_text = format!(
        "Projects - {} | Page {}/{}{}",
        type_name, state.current_page, state.total_pages, filter_str
    );
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let visible_row_limit = 10usize;
    let visible_start = state
        .selected_row
        .saturating_sub(visible_row_limit.saturating_sub(1));

    let rows: Vec<Row> = state
        .projects
        .data
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(visible_row_limit)
        .map(|(i, p)| {
            let style = if i == state.selected_row {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let name = if p.name.len() > 12 {
                format!("{}...", &p.name[..9])
            } else {
                p.name.clone()
            };

            let summary = if p.summary.len() > 20 {
                format!("{}...", &p.summary[..17])
            } else {
                p.summary.clone()
            };

            let downloads = format_number(p.downloads);

            let updated = p
                .updated_at
                .as_ref()
                .map(|d| if d.len() >= 10 { &d[..10] } else { d.as_str() })
                .unwrap_or("N/A");

            let created = if p.created_at.len() >= 10 {
                &p.created_at[..10]
            } else {
                p.created_at.as_str()
            };

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
        Row::new(vec![
            "Name",
            "Summary",
            "Tags",
            "Downloads",
            "Status",
            "Updated",
            "Created",
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1),
    )
    .block(Block::default().borders(Borders::ALL).title(" Projects "));

    let max_table_height = 14;
    let table_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width,
        height: chunks[1].height.min(max_table_height),
    };
    f.render_widget(table, table_area);

    let help = Paragraph::new(
        "n: Next | p: Prev | f: Filter | s: Search | o: Sort | r: Reset | t: Back | q: Quit",
    )
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    render_status_bar(f, chunks[3], state);
}

/// Render the tag filter screen with side-by-side scrollable tables.
fn render_tag_filter(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(f.area());

    let title = Paragraph::new("Select Tags to Filter")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let project_visible_rows = calculate_visible_rows(table_chunks[0].height);
    let version_visible_rows = calculate_visible_rows(table_chunks[1].height);

    let mut project_rows: Vec<Row> = Vec::new();
    let scroll = state.project_tag_scroll;
    let visible_end = scroll + project_visible_rows;

    for (i, (tag, depth)) in state.flattened_tags.iter().enumerate() {
        if i >= scroll && i < visible_end {
            let style = if state.tag_table_focus == TagTableFocus::ProjectTags
                && i == state.project_tag_selection
            {
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

            let display_name = format!("{}{} {} | {}", indent, checkbox, tag.slug, tag.name);
            project_rows.push(Row::new(vec![Cell::from(display_name)]).style(style));
        }
    }

    let project_table = Table::new(project_rows, [Constraint::Min(30)]).block(
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

    let mut version_rows: Vec<Row> = Vec::new();
    let scroll = state.version_tag_scroll;
    let visible_end = scroll + version_visible_rows;

    for (i, (tag, depth)) in state.flattened_version_tags.iter().enumerate() {
        if i >= scroll && i < visible_end {
            let style = if state.tag_table_focus == TagTableFocus::VersionTags
                && i == state.version_tag_selection
            {
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

            let display_name = format!("{}{} {} | {}", indent, checkbox, tag.slug, tag.name);
            version_rows.push(Row::new(vec![Cell::from(display_name)]).style(style));
        }
    }

    let version_table = Table::new(version_rows, [Constraint::Min(30)]).block(
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

    let help = Paragraph::new(
        "Tab: Switch Table | Up/Down: Navigate | Enter: Toggle | Esc: Apply | r: Reset | q: Quit",
    )
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    render_status_bar(f, chunks[3], state);
}

/// Render the search input screen.
fn render_search_input(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(f.area());

    let title = Paragraph::new("Search Projects")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .title(" Search Query ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_text = Paragraph::new(state.search_query.as_str())
        .style(Style::default().fg(Color::White))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    let cursor_x = chunks[1].x + state.search_cursor_position as u16 + 1;
    let cursor_y = chunks[1].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    let help = Paragraph::new("Enter: Search | Esc: Cancel | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    render_status_bar(f, chunks[4], state);
}

/// Render the sort options screen.
fn render_sort_options(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(f.area());

    let title = Paragraph::new("Sort Options")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

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
            Row::new(vec![Cell::from(format!("{}{}", marker, name))]).style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Length(20)]).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Sort Field "),
    );
    f.render_widget(table, chunks[1]);

    let help = Paragraph::new("Enter: Select | d: Toggle Direction | Esc: Apply | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    render_status_bar(f, chunks[3], state);
}

/// Render the status bar at the bottom.
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

fn calculate_visible_rows(table_height: u16) -> usize {
    table_height.saturating_sub(4) as usize
}

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
