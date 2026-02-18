use std::collections::HashSet;

use crate::{
    HubClient, ListProjectsParams, PaginatedResponse, Project, ProjectTag, ProjectType,
    ProjectVersionTag,
};

/// Represents which tag table is currently focused.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TagTableFocus {
    /// Project tags table is focused.
    ProjectTags,
    /// Version tags table is focused.
    VersionTags,
}

/// Represents the current screen being displayed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AppScreen {
    /// URL input screen - user enters API base URL.
    UrlInput,
    /// Loading project types from API.
    LoadingTypes,
    /// Project type selection screen.
    TypeSelection,
    /// Loading projects from API.
    LoadingProjects,
    /// Project table screen with pagination.
    ProjectTable,
    /// Tag filter selection screen.
    TagFilter,
    /// Loading tags from API.
    LoadingTags,
    /// Search input screen.
    SearchInput,
    /// Sort options screen.
    SortOptions,
}

/// Sort field options.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SortField {
    Downloads,
    Name,
    UpdatedAt,
    CreatedAt,
}

impl SortField {
    pub(crate) fn to_api_string(&self) -> &'static str {
        match self {
            SortField::Downloads => "downloads",
            SortField::Name => "name",
            SortField::UpdatedAt => "updated_at",
            SortField::CreatedAt => "created_at",
        }
    }
}

/// Sort direction options.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub(crate) fn to_api_string(&self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }
}

/// Main application state.
pub struct AppState {
    /// Current screen being displayed.
    pub(crate) screen: AppScreen,
    /// API base URL entered by user.
    pub(crate) api_url: String,
    /// Cursor position in URL input.
    pub(crate) cursor_position: usize,
    /// Initialized API client.
    pub(crate) client: Option<HubClient>,
    /// List of available project types.
    pub(crate) project_types: Vec<ProjectType>,
    /// Currently selected project type index.
    pub(crate) selected_type_index: usize,
    /// Paginated response of projects.
    pub(crate) projects: PaginatedResponse<Project>,
    /// Current page number (1-indexed).
    pub(crate) current_page: u32,
    /// Total number of pages.
    pub(crate) total_pages: u32,
    /// Currently selected row in table.
    pub(crate) selected_row: usize,
    /// Error message to display.
    pub(crate) error_message: Option<String>,
    /// Should the application quit?
    pub should_quit: bool,

    // Filter state
    /// List of available tags for filtering.
    pub(crate) available_tags: Vec<ProjectTag>,
    /// Set of selected tag slugs for filtering.
    pub(crate) selected_tags: HashSet<String>,
    /// Currently selected tag table.
    pub(crate) tag_table_focus: TagTableFocus,
    /// Currently selected index within project tags table.
    pub(crate) project_tag_selection: usize,
    /// Scroll offset for project tags table.
    pub(crate) project_tag_scroll: usize,
    /// Currently selected index within version tags table.
    pub(crate) version_tag_selection: usize,
    /// Scroll offset for version tags table.
    pub(crate) version_tag_scroll: usize,
    /// Flattened list of (tag, depth) for display.
    pub(crate) flattened_tags: Vec<(ProjectTag, usize)>,
    /// List of available version tags for filtering.
    pub(crate) available_version_tags: Vec<ProjectVersionTag>,
    /// Set of selected version tag slugs for filtering.
    pub(crate) selected_version_tags: HashSet<String>,
    /// Flattened list of version tags for display.
    pub(crate) flattened_version_tags: Vec<(ProjectVersionTag, usize)>,

    // Search state
    /// Search query string.
    pub(crate) search_query: String,
    /// Cursor position in search input.
    pub(crate) search_cursor_position: usize,

    // Sort state
    /// Current sort field.
    pub(crate) sort_field: SortField,
    /// Current sort direction.
    pub(crate) sort_direction: SortDirection,
    /// Currently selected index in sort options.
    pub(crate) sort_selection_index: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: AppScreen::UrlInput,
            api_url: "https://hub01-shop.srgnis.com/api".to_string(),
            cursor_position: "https://hub01-shop.srgnis.com/api".len(),
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
            available_tags: Vec::new(),
            selected_tags: HashSet::new(),
            tag_table_focus: TagTableFocus::ProjectTags,
            project_tag_selection: 0,
            project_tag_scroll: 0,
            version_tag_selection: 0,
            version_tag_scroll: 0,
            flattened_tags: Vec::new(),
            available_version_tags: Vec::new(),
            selected_version_tags: HashSet::new(),
            flattened_version_tags: Vec::new(),
            search_query: String::new(),
            search_cursor_position: 0,
            sort_field: SortField::Downloads,
            sort_direction: SortDirection::Desc,
            sort_selection_index: 0,
        }
    }
}

impl AppState {
    /// Create a new application state with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the API client with the current URL.
    pub(crate) fn init_client(&mut self) -> Result<(), String> {
        match HubClient::new(&self.api_url, None) {
            Ok(client) => {
                self.client = Some(client);
                Ok(())
            }
            Err(e) => Err(format!("Failed to create client: {e}")),
        }
    }

    /// Fetch project types from the API.
    pub(crate) fn fetch_project_types(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        match client.project_types().list() {
            Ok(types) => {
                self.project_types = types;
                if self.project_types.is_empty() {
                    return Err("No project types found".to_string());
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to fetch project types: {e}")),
        }
    }

    /// Fetch projects for the selected type.
    pub(crate) fn fetch_projects(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        let project_type = self
            .project_types
            .get(self.selected_type_index)
            .ok_or("No project type selected")?;

        let tags: Vec<String> = self.selected_tags.iter().cloned().collect();
        let version_tags: Vec<String> = self.selected_version_tags.iter().cloned().collect();
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
            version_tags: if version_tags.is_empty() {
                None
            } else {
                Some(version_tags)
            },
            search,
            order_by: Some(self.sort_field.to_api_string().to_string()),
            order_direction: Some(self.sort_direction.to_api_string().to_string()),
            ..Default::default()
        };

        match client.projects().list(&params) {
            Ok(response) => {
                self.projects = response;
                if let Some(meta) = &self.projects.meta {
                    if let Some(total) = meta.get("total").and_then(|t| t.as_u64()) {
                        let per_page = meta.get("per_page").and_then(|p| p.as_u64()).unwrap_or(10);
                        self.total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
                    }
                }
                self.selected_row = 0;
                Ok(())
            }
            Err(e) => Err(format!("Failed to fetch projects: {e}")),
        }
    }

    /// Fetch tags for the selected project type.
    pub(crate) fn fetch_tags(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        let project_type = self
            .project_types
            .get(self.selected_type_index)
            .ok_or("No project type selected")?;

        match client.tags().list_project_tags(false, Some(&project_type.slug)) {
            Ok(tags) => {
                self.available_tags = tags;
            }
            Err(e) => return Err(format!("Failed to fetch project tags: {e}")),
        }

        match client.tags().list_version_tags(false, Some(&project_type.slug)) {
            Ok(tags) => {
                self.available_version_tags = tags;
            }
            Err(e) => return Err(format!("Failed to fetch version tags: {e}")),
        }

        self.flatten_tags();
        self.project_tag_selection = 0;
        self.version_tag_selection = 0;
        Ok(())
    }

    /// Flatten the hierarchical tags into a display list with subtags under their parents.
    pub(crate) fn flatten_tags(&mut self) {
        self.flattened_tags.clear();
        for tag in &self.available_tags {
            self.flattened_tags.push((tag.clone(), 0));
            for sub_tag in &tag.sub_tags {
                self.flattened_tags.push((sub_tag.clone(), 1));
            }
        }

        self.flattened_version_tags.clear();
        for tag in &self.available_version_tags {
            self.flattened_version_tags.push((tag.clone(), 0));
            for sub_tag in &tag.sub_tags {
                self.flattened_version_tags.push((sub_tag.clone(), 1));
            }
        }
    }

    /// Reset all filters to default state.
    pub(crate) fn reset_filters(&mut self) {
        self.selected_tags.clear();
        self.selected_version_tags.clear();
        self.search_query.clear();
        self.search_cursor_position = 0;
        self.sort_field = SortField::Downloads;
        self.sort_direction = SortDirection::Desc;
        self.current_page = 1;
    }

    /// Clear any error message.
    pub(crate) fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set an error message.
    pub(crate) fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }
}
