use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use reqwest::blocking::Client as HttpClient;

use hub01_client::{
    HubClient, ListProjectsParams, ListVersionsParams, PaginatedResponse, Project, ProjectTag,
    ProjectType, ProjectVersion, ProjectVersionTag,
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
    /// Loading selected project details and its first versions page.
    LoadingProjectDetails,
    /// Selected project details with versions table.
    ProjectDetails,
    /// Loading versions page for selected project.
    LoadingProjectVersions,
    /// Loading selected project version details.
    LoadingVersionDetails,
    /// Selected project version details with downloadable files.
    VersionDetails,
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
    /// Non-error status message (e.g. successful downloads).
    pub(crate) success_message: Option<String>,
    /// Should the application quit?
    pub should_quit: bool,

    // Project details state
    /// Currently selected project details.
    pub(crate) selected_project: Option<Project>,
    /// Paginated versions of the selected project.
    pub(crate) project_versions: PaginatedResponse<ProjectVersion>,
    /// Current selected project's versions page (1-indexed).
    pub(crate) versions_current_page: u32,
    /// Total selected project's versions pages.
    pub(crate) versions_total_pages: u32,
    /// Selected row in versions table.
    pub(crate) selected_version_row: usize,

    // Version details state
    /// Currently selected version details.
    pub(crate) selected_version: Option<ProjectVersion>,
    /// Selected row in version files table.
    pub(crate) selected_file_row: usize,

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
            success_message: None,
            should_quit: false,
            selected_project: None,
            project_versions: PaginatedResponse {
                data: Vec::new(),
                meta: None,
                links: None,
            },
            versions_current_page: 1,
            versions_total_pages: 1,
            selected_version_row: 0,
            selected_version: None,
            selected_file_row: 0,
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
                self.total_pages = Self::calculate_total_pages(self.projects.meta.as_ref(), 10);
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

    /// Fetch details for the currently selected project in project table.
    pub(crate) fn fetch_selected_project_details(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        let project = self
            .projects
            .data
            .get(self.selected_row)
            .ok_or("No project selected")?;

        match client.projects().get(&project.slug) {
            Ok(project_details) => {
                self.selected_project = Some(project_details);
                self.project_versions = PaginatedResponse {
                    data: Vec::new(),
                    meta: None,
                    links: None,
                };
                self.selected_version = None;
                self.selected_file_row = 0;
                self.selected_version_row = 0;
                self.versions_current_page = 1;
                self.versions_total_pages = 1;
                Ok(())
            }
            Err(e) => Err(format!("Failed to fetch project details: {e}")),
        }
    }

    /// Fetch paginated versions for currently selected project.
    pub(crate) fn fetch_project_versions(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        let project = self
            .selected_project
            .as_ref()
            .ok_or("No selected project details loaded")?;

        let params = ListVersionsParams {
            per_page: 10,
            page: self.versions_current_page,
            ..Default::default()
        };

        match client.versions().list(&project.slug, &params) {
            Ok(response) => {
                self.project_versions = response;
                self.versions_total_pages =
                    Self::calculate_total_pages(self.project_versions.meta.as_ref(), 10);
                self.selected_version_row = 0;
                Ok(())
            }
            Err(e) => Err(format!("Failed to fetch project versions: {e}")),
        }
    }

    /// Fetch selected version details from versions table selection.
    pub(crate) fn fetch_selected_version_details(&mut self) -> Result<(), String> {
        let client = self.client.as_ref().ok_or("Client not initialized")?;
        let project = self
            .selected_project
            .as_ref()
            .ok_or("No selected project details loaded")?;
        let version = self
            .project_versions
            .data
            .get(self.selected_version_row)
            .ok_or("No version selected")?;

        match client.versions().get(&project.slug, &version.version) {
            Ok(version_details) => {
                self.selected_version = Some(version_details);
                self.selected_file_row = 0;
                Ok(())
            }
            Err(e) => Err(format!("Failed to fetch version details: {e}")),
        }
    }

    /// Download currently selected file in version details screen to current working directory.
    pub(crate) fn download_selected_file(&mut self) -> Result<String, String> {
        let _client = self.client.as_ref().ok_or("Client not initialized")?;
        let version = self
            .selected_version
            .as_ref()
            .ok_or("No version details loaded")?;
        let file = version
            .files
            .get(self.selected_file_row)
            .ok_or("No file selected")?;

        let http = HttpClient::new();
        let response = http
            .get(&file.url)
            .send()
            .map_err(|e| format!("Failed to download file: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let reason = response
                .text()
                .unwrap_or_else(|_| format!("HTTP {}", status.as_u16()));
            return Err(format!("Failed to download file: {reason}"));
        }

        let bytes = response
            .bytes()
            .map_err(|e| format!("Failed to read downloaded bytes: {e}"))?;

        let target_path = ensure_unique_file_path(&file.name);
        fs::write(&target_path, bytes)
            .map_err(|e| format!("Failed to write file '{}': {e}", target_path.display()))?;

        let filename = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&file.name)
            .to_string();
        Ok(filename)
    }

    /// Clear any error message.
    pub(crate) fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set an error message.
    pub(crate) fn set_error(&mut self, message: String) {
        self.success_message = None;
        self.error_message = Some(message);
    }

    /// Clear non-error success message.
    pub(crate) fn clear_success(&mut self) {
        self.success_message = None;
    }

    /// Set a non-error status message.
    pub(crate) fn set_success(&mut self, message: String) {
        self.error_message = None;
        self.success_message = Some(message);
    }

    fn calculate_total_pages(meta: Option<&serde_json::Value>, default_per_page: u64) -> u32 {
        let Some(meta) = meta else {
            return 1;
        };

        let total = meta.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
        let per_page = meta
            .get("per_page")
            .and_then(|p| p.as_u64())
            .unwrap_or(default_per_page)
            .max(1);

        let pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        pages.max(1)
    }
}

fn ensure_unique_file_path(file_name: &str) -> PathBuf {
    let mut candidate = PathBuf::from(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let ext = Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    for i in 1..=9_999 {
        let next_name = if ext.is_empty() {
            format!("{stem}-{i}")
        } else {
            format!("{stem}-{i}.{ext}")
        };
        candidate = PathBuf::from(&next_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from(format!("{stem}-overflow"))
}
