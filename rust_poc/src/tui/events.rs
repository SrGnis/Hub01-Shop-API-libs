use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::state::{AppScreen, AppState, SortDirection, SortField, TagTableFocus};

/// Main event handler that dispatches to the appropriate screen handler.
pub fn handle_event(event: Event, state: &mut AppState) {
    if let Event::Key(key_event) = event {
        if key_event.code == KeyCode::Char('q')
            && state.screen != AppScreen::UrlInput
            && key_event.modifiers == KeyModifiers::NONE
        {
            state.should_quit = true;
            return;
        }

        match state.screen {
            AppScreen::UrlInput => handle_url_input(key_event, state),
            AppScreen::TypeSelection => handle_type_selection(key_event, state),
            AppScreen::ProjectTable => handle_project_table(key_event, state),
            AppScreen::ProjectDetails => handle_project_details(key_event, state),
            AppScreen::VersionDetails => handle_version_details(key_event, state),
            AppScreen::TagFilter => handle_tag_filter(key_event, state),
            AppScreen::SearchInput => handle_search_input(key_event, state),
            AppScreen::SortOptions => handle_sort_options(key_event, state),
            _ => {}
        }
    }
}

fn handle_url_input(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Char(c) => {
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
            state.screen = AppScreen::LoadingTags;
        }
        KeyCode::Char('s') => {
            state.screen = AppScreen::SearchInput;
        }
        KeyCode::Char('o') => {
            state.sort_selection_index = match state.sort_field {
                SortField::Downloads => 0,
                SortField::Name => 1,
                SortField::UpdatedAt => 2,
                SortField::CreatedAt => 3,
            };
            state.screen = AppScreen::SortOptions;
        }
        KeyCode::Char('r') => {
            state.reset_filters();
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Enter => {
            if !state.projects.data.is_empty() {
                state.screen = AppScreen::LoadingProjectDetails;
                state.clear_error();
                state.clear_success();
            } else {
                state.set_error("No project selected".to_string());
            }
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        _ => {}
    }
}

fn handle_project_details(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Up => {
            if state.selected_version_row > 0 {
                state.selected_version_row -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected_version_row < state.project_versions.data.len().saturating_sub(1) {
                state.selected_version_row += 1;
            }
        }
        KeyCode::Char('n') => {
            if state.versions_current_page < state.versions_total_pages {
                state.versions_current_page += 1;
                state.screen = AppScreen::LoadingProjectVersions;
            }
        }
        KeyCode::Char('p') => {
            if state.versions_current_page > 1 {
                state.versions_current_page -= 1;
                state.screen = AppScreen::LoadingProjectVersions;
            }
        }
        KeyCode::Enter => {
            if !state.project_versions.data.is_empty() {
                state.screen = AppScreen::LoadingVersionDetails;
                state.clear_error();
                state.clear_success();
            } else {
                state.set_error("No version selected".to_string());
            }
        }
        KeyCode::Esc => {
            state.screen = AppScreen::ProjectTable;
            state.clear_error();
        }
        _ => {}
    }
}

fn handle_version_details(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Up => {
            if state.selected_file_row > 0 {
                state.selected_file_row -= 1;
            }
        }
        KeyCode::Down => {
            let total_files = state
                .selected_version
                .as_ref()
                .map(|v| v.files.len())
                .unwrap_or(0);
            if state.selected_file_row < total_files.saturating_sub(1) {
                state.selected_file_row += 1;
            }
        }
        KeyCode::Enter => match state.download_selected_file() {
            Ok(saved_name) => {
                state.set_success(format!("Downloaded file: {saved_name}"));
            }
            Err(e) => {
                state.set_error(e);
            }
        },
        KeyCode::Esc => {
            state.screen = AppScreen::ProjectDetails;
            state.clear_error();
        }
        _ => {}
    }
}

fn handle_tag_filter(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Up => match state.tag_table_focus {
            TagTableFocus::ProjectTags => {
                if state.project_tag_selection > 0 {
                    state.project_tag_selection -= 1;
                    ensure_selection_visible(
                        state.project_tag_selection,
                        &mut state.project_tag_scroll,
                        10,
                        state.flattened_tags.len(),
                    );
                }
            }
            TagTableFocus::VersionTags => {
                if state.version_tag_selection > 0 {
                    state.version_tag_selection -= 1;
                    ensure_selection_visible(
                        state.version_tag_selection,
                        &mut state.version_tag_scroll,
                        10,
                        state.flattened_version_tags.len(),
                    );
                }
            }
        },
        KeyCode::Down => match state.tag_table_focus {
            TagTableFocus::ProjectTags => {
                if state.project_tag_selection < state.flattened_tags.len().saturating_sub(1) {
                    state.project_tag_selection += 1;
                    ensure_selection_visible(
                        state.project_tag_selection,
                        &mut state.project_tag_scroll,
                        10,
                        state.flattened_tags.len(),
                    );
                }
            }
            TagTableFocus::VersionTags => {
                if state.version_tag_selection < state.flattened_version_tags.len().saturating_sub(1)
                {
                    state.version_tag_selection += 1;
                    ensure_selection_visible(
                        state.version_tag_selection,
                        &mut state.version_tag_scroll,
                        10,
                        state.flattened_version_tags.len(),
                    );
                }
            }
        },
        KeyCode::Tab => {
            state.tag_table_focus = match state.tag_table_focus {
                TagTableFocus::ProjectTags => TagTableFocus::VersionTags,
                TagTableFocus::VersionTags => TagTableFocus::ProjectTags,
            };
        }
        KeyCode::Enter => match state.tag_table_focus {
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
                if let Some((tag, _depth)) = state
                    .flattened_version_tags
                    .get(state.version_tag_selection)
                {
                    if state.selected_version_tags.contains(&tag.slug) {
                        state.selected_version_tags.remove(&tag.slug);
                    } else {
                        state.selected_version_tags.insert(tag.slug.clone());
                    }
                }
            }
        },
        KeyCode::Char('r') => {
            state.selected_tags.clear();
            state.selected_version_tags.clear();
            state.project_tag_selection = 0;
            state.version_tag_selection = 0;
            state.project_tag_scroll = 0;
            state.version_tag_scroll = 0;
            state.tag_table_focus = TagTableFocus::ProjectTags;
        }
        KeyCode::Esc => {
            state.current_page = 1;
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        _ => {}
    }
}

fn handle_search_input(event: KeyEvent, state: &mut AppState) {
    match event.code {
        KeyCode::Char(c) if c != 'q' => {
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
            state.current_page = 1;
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Esc => {
            state.screen = AppScreen::ProjectTable;
        }
        _ => {}
    }
}

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
            state.sort_direction = match state.sort_direction {
                SortDirection::Asc => SortDirection::Desc,
                SortDirection::Desc => SortDirection::Asc,
            };
        }
        KeyCode::Esc => {
            state.current_page = 1;
            state.screen = AppScreen::LoadingProjects;
        }
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        _ => {}
    }
}

fn ensure_selection_visible(selection: usize, scroll: &mut usize, visible_rows: usize, total_rows: usize) {
    if visible_rows == 0 {
        return;
    }

    if selection < *scroll {
        *scroll = selection;
    } else if selection >= *scroll + visible_rows {
        *scroll = selection - visible_rows.saturating_sub(1);
    }

    *scroll = (*scroll).clamp(0, total_rows.saturating_sub(1));
}
