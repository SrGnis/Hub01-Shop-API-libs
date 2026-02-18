use super::state::{AppScreen, AppState};

/// Process the current state (e.g., fetch data during loading screens).
pub fn process_state(state: &mut AppState) {
    match state.screen {
        AppScreen::LoadingTypes => match state.fetch_project_types() {
            Ok(()) => {
                state.screen = AppScreen::TypeSelection;
                state.selected_type_index = 0;
            }
            Err(e) => {
                state.set_error(e);
                state.screen = AppScreen::UrlInput;
            }
        },
        AppScreen::LoadingProjects => match state.fetch_projects() {
            Ok(()) => {
                state.screen = AppScreen::ProjectTable;
            }
            Err(e) => {
                state.set_error(e);
                state.screen = AppScreen::TypeSelection;
            }
        },
        AppScreen::LoadingTags => match state.fetch_tags() {
            Ok(()) => {
                state.screen = AppScreen::TagFilter;
            }
            Err(e) => {
                state.set_error(e);
                state.screen = AppScreen::ProjectTable;
            }
        },
        AppScreen::LoadingProjectDetails => match state.fetch_selected_project_details() {
            Ok(()) => {
                state.screen = AppScreen::LoadingProjectVersions;
            }
            Err(e) => {
                state.set_error(e);
                state.screen = AppScreen::ProjectTable;
            }
        },
        AppScreen::LoadingProjectVersions => match state.fetch_project_versions() {
            Ok(()) => {
                state.screen = AppScreen::ProjectDetails;
            }
            Err(e) => {
                state.set_error(e);
                state.screen = AppScreen::ProjectDetails;
            }
        },
        AppScreen::LoadingVersionDetails => match state.fetch_selected_version_details() {
            Ok(()) => {
                state.screen = AppScreen::VersionDetails;
            }
            Err(e) => {
                state.set_error(e);
                state.screen = AppScreen::ProjectDetails;
            }
        },
        _ => {}
    }
}
