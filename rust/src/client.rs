use reqwest::blocking::{multipart, Client, Response};
use serde::Deserialize;

use crate::error::{HubApiError, Result};
use crate::models::*;

// ---------------------------------------------------------------------------
// Helper: unwrap `{ "data": ... }` wrapper used by most endpoints
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct DataWrapper<T> {
    data: T,
}

// ---------------------------------------------------------------------------
// Internal response handling
// ---------------------------------------------------------------------------

/// Shared logic for building a configured [`Client`] and making requests.
struct BaseClient {
    base_url: String,
    http: Client,
}

impl BaseClient {
    fn new(base_url: &str, token: Option<&str>) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        if let Some(tok) = token {
            let val =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {tok}")).map_err(|e| {
                    HubApiError::Api {
                        status: 0,
                        message: format!("Invalid token header value: {e}"),
                    }
                })?;
            headers.insert(reqwest::header::AUTHORIZATION, val);
        }

        let http = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http,
        })
    }

    /// Build the full URL for a given endpoint.
    fn url(&self, endpoint: &str) -> String {
        format!("{}{endpoint}", self.base_url)
    }

    /// Send a request and handle status-code → error mapping.
    fn handle_response(&self, response: Response) -> Result<Option<serde_json::Value>> {
        let status = response.status().as_u16();

        if status == 204 {
            return Ok(None);
        }

        // Try to parse JSON body; fall back to empty object on failure.
        let data: serde_json::Value = response
            .json()
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

        if (200..300).contains(&status) {
            return Ok(Some(data));
        }

        let msg = data
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        match status {
            401 => Err(HubApiError::Authentication {
                message: if msg.is_empty() {
                    "Unauthenticated".into()
                } else {
                    msg
                },
            }),
            403 => Err(HubApiError::PermissionDenied {
                message: if msg.is_empty() {
                    "Permission denied".into()
                } else {
                    msg
                },
            }),
            404 => Err(HubApiError::NotFound {
                message: if msg.is_empty() {
                    "Not found".into()
                } else {
                    msg
                },
            }),
            422 => Err(HubApiError::Validation {
                message: if msg.is_empty() {
                    "Validation error".into()
                } else {
                    msg
                },
                errors: data.get("errors").cloned(),
            }),
            _ => Err(HubApiError::Api {
                status,
                message: if msg.is_empty() {
                    format!("HTTP {status}")
                } else {
                    msg
                },
            }),
        }
    }

    // ---- convenience wrappers for common HTTP verbs -----------------------

    fn get(&self, endpoint: &str, query: &[(String, String)]) -> Result<Option<serde_json::Value>> {
        let resp = self.http.get(self.url(endpoint)).query(query).send()?;
        self.handle_response(resp)
    }

    fn post_multipart(
        &self,
        endpoint: &str,
        form: multipart::Form,
    ) -> Result<Option<serde_json::Value>> {
        let resp = self.http.post(self.url(endpoint)).multipart(form).send()?;
        self.handle_response(resp)
    }

    fn delete(&self, endpoint: &str) -> Result<Option<serde_json::Value>> {
        let resp = self.http.delete(self.url(endpoint)).send()?;
        self.handle_response(resp)
    }
}

// ---------------------------------------------------------------------------
// Public client
// ---------------------------------------------------------------------------

/// Main entry point for interacting with the Hub01 Shop API.
///
/// ```no_run
/// use hub01_client::HubClient;
///
/// let client = HubClient::new("https://hub01-shop.srgnis.com/api", None).unwrap();
/// let types = client.project_types().list().unwrap();
/// for t in &types {
///     println!("{}: {}", t.name, t.slug);
/// }
/// ```
pub struct HubClient {
    base: BaseClient,
}

impl HubClient {
    /// Create a new client.
    ///
    /// * `base_url` – API root, e.g. `https://hub01-shop.srgnis.com/api`
    /// * `token`    – optional bearer token for authenticated operations
    pub fn new(base_url: &str, token: Option<&str>) -> Result<Self> {
        Ok(Self {
            base: BaseClient::new(base_url, token)?,
        })
    }

    /// Validate the configured API token.
    pub fn test_token(&self) -> Result<serde_json::Value> {
        self.base
            .get("/test-token", &[])?
            .ok_or_else(|| HubApiError::Api {
                status: 0,
                message: "Empty response from test-token".into(),
            })
    }

    // -- sub-client accessors ------------------------------------------------

    pub fn project_types(&self) -> ProjectTypesClient<'_> {
        ProjectTypesClient { base: &self.base }
    }

    pub fn projects(&self) -> ProjectsClient<'_> {
        ProjectsClient { base: &self.base }
    }

    pub fn versions(&self) -> ProjectVersionsClient<'_> {
        ProjectVersionsClient { base: &self.base }
    }

    pub fn tags(&self) -> TagsClient<'_> {
        TagsClient { base: &self.base }
    }

    pub fn users(&self) -> UsersClient<'_> {
        UsersClient { base: &self.base }
    }
}

// ===========================================================================
// Sub-clients
// ===========================================================================

// ---- Project Types --------------------------------------------------------

pub struct ProjectTypesClient<'a> {
    base: &'a BaseClient,
}

impl ProjectTypesClient<'_> {
    /// List all project types.
    pub fn list(&self) -> Result<Vec<ProjectType>> {
        let data = self.base.get("/v1/project_types", &[])?;
        let wrapper: DataWrapper<Vec<ProjectType>> =
            serde_json::from_value(data.unwrap_or_default()).map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// Get a single project type by slug.
    pub fn get(&self, slug: &str) -> Result<ProjectType> {
        let data = self.base.get(&format!("/v1/project_type/{slug}"), &[])?;
        let wrapper: DataWrapper<ProjectType> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }
}

// ---- Projects -------------------------------------------------------------

pub struct ProjectsClient<'a> {
    base: &'a BaseClient,
}

/// Parameters for listing / searching projects.
pub struct ListProjectsParams {
    pub project_type: Option<String>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub version_tags: Option<Vec<String>>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub per_page: u32,
    pub page: u32,
    pub release_date_period: Option<String>,
    pub release_date_start: Option<String>,
    pub release_date_end: Option<String>,
}

impl Default for ListProjectsParams {
    fn default() -> Self {
        Self {
            project_type: Some("mod".into()),
            search: None,
            tags: None,
            version_tags: None,
            order_by: Some("downloads".into()),
            order_direction: Some("desc".into()),
            per_page: 10,
            page: 1,
            release_date_period: Some("all".into()),
            release_date_start: None,
            release_date_end: None,
        }
    }
}

impl ProjectsClient<'_> {
    /// List / search projects with pagination.
    pub fn list(&self, params: &ListProjectsParams) -> Result<PaginatedResponse<Project>> {
        let mut query: Vec<(String, String)> = Vec::new();

        if let Some(ref v) = params.project_type {
            query.push(("project_type".into(), v.clone()));
        }
        if let Some(ref v) = params.search {
            query.push(("search".into(), v.clone()));
        }
        if let Some(ref tags) = params.tags {
            for t in tags {
                query.push(("tags[]".into(), t.clone()));
            }
        }
        if let Some(ref tags) = params.version_tags {
            for t in tags {
                query.push(("version_tags[]".into(), t.clone()));
            }
        }
        if let Some(ref v) = params.order_by {
            query.push(("order_by".into(), v.clone()));
        }
        if let Some(ref v) = params.order_direction {
            query.push(("order_direction".into(), v.clone()));
        }
        query.push(("per_page".into(), params.per_page.to_string()));
        query.push(("page".into(), params.page.to_string()));
        if let Some(ref v) = params.release_date_period {
            query.push(("release_date_period".into(), v.clone()));
        }
        if let Some(ref v) = params.release_date_start {
            query.push(("release_date_start".into(), v.clone()));
        }
        if let Some(ref v) = params.release_date_end {
            query.push(("release_date_end".into(), v.clone()));
        }

        let data = self.base.get("/v1/projects", &query)?;
        let resp: PaginatedResponse<Project> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(resp)
    }

    /// Get a single project by slug.
    pub fn get(&self, slug: &str) -> Result<Project> {
        let data = self.base.get(&format!("/v1/project/{slug}"), &[])?;
        let wrapper: DataWrapper<Project> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }
}

// ---- Project Versions -----------------------------------------------------

pub struct ProjectVersionsClient<'a> {
    base: &'a BaseClient,
}

/// Parameters for listing project versions.
pub struct ListVersionsParams {
    pub tags: Option<Vec<String>>,
    pub order_by: String,
    pub order_direction: String,
    pub per_page: u32,
    pub page: u32,
}

impl Default for ListVersionsParams {
    fn default() -> Self {
        Self {
            tags: None,
            order_by: "downloads".into(),
            order_direction: "desc".into(),
            per_page: 10,
            page: 1,
        }
    }
}

/// Parameters for creating a new project version.
pub struct CreateVersionParams {
    pub name: String,
    pub version: String,
    pub release_type: String,
    pub release_date: String,
    pub changelog: String,
    pub tags: Option<Vec<String>>,
    pub dependencies: Option<Vec<Dependency>>,
}

/// Parameters for updating an existing project version.
#[derive(Default)]
pub struct UpdateVersionParams {
    pub name: Option<String>,
    pub version_new: Option<String>,
    pub release_type: Option<String>,
    pub release_date: Option<String>,
    pub changelog: Option<String>,
    pub tags: Option<Vec<String>>,
    pub files_to_remove: Option<Vec<String>>,
    pub clean_existing_files: bool,
    pub dependencies: Option<Vec<Dependency>>,
}

/// A dependency descriptor used when creating/updating versions.
pub struct Dependency {
    pub project: String,
    pub version: String,
    pub dep_type: String,
    pub external: bool,
}

impl ProjectVersionsClient<'_> {
    /// List all versions of a project.
    pub fn list(
        &self,
        slug: &str,
        params: &ListVersionsParams,
    ) -> Result<PaginatedResponse<ProjectVersion>> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(ref tags) = params.tags {
            for t in tags {
                query.push(("tags[]".into(), t.clone()));
            }
        }
        query.push(("order_by".into(), params.order_by.clone()));
        query.push(("order_direction".into(), params.order_direction.clone()));
        query.push(("per_page".into(), params.per_page.to_string()));
        query.push(("page".into(), params.page.to_string()));

        let data = self
            .base
            .get(&format!("/v1/project/{slug}/versions"), &query)?;
        let resp: PaginatedResponse<ProjectVersion> =
            serde_json::from_value(data.unwrap_or_default()).map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(resp)
    }

    /// Get a single project version.
    pub fn get(&self, slug: &str, version: &str) -> Result<ProjectVersion> {
        let data = self
            .base
            .get(&format!("/v1/project/{slug}/version/{version}"), &[])?;
        let wrapper: DataWrapper<ProjectVersion> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// Create a new project version with file uploads.
    ///
    /// `files` is a list of `(filename, bytes)` tuples.
    pub fn create(
        &self,
        slug: &str,
        params: &CreateVersionParams,
        files: &[(&str, Vec<u8>)],
    ) -> Result<ProjectVersion> {
        let mut form = multipart::Form::new()
            .text("name", params.name.clone())
            .text("version", params.version.clone())
            .text("release_type", params.release_type.clone())
            .text("release_date", params.release_date.clone())
            .text("changelog", params.changelog.clone());

        if let Some(ref tags) = params.tags {
            for t in tags {
                form = form.text("tags[]", t.clone());
            }
        }

        if let Some(ref deps) = params.dependencies {
            for (i, dep) in deps.iter().enumerate() {
                form = form.text(format!("dependencies[{i}][project]"), dep.project.clone());
                form = form.text(format!("dependencies[{i}][version]"), dep.version.clone());
                form = form.text(format!("dependencies[{i}][type]"), dep.dep_type.clone());
                form = form.text(
                    format!("dependencies[{i}][external]"),
                    if dep.external { "1" } else { "0" }.to_string(),
                );
            }
        }

        for (filename, bytes) in files {
            let part = multipart::Part::bytes(bytes.clone())
                .file_name(filename.to_string())
                .mime_str("application/octet-stream")
                .map_err(|e| HubApiError::Api {
                    status: 0,
                    message: format!("Invalid MIME type: {e}"),
                })?;
            form = form.part("files[]", part);
        }

        let data = self
            .base
            .post_multipart(&format!("/v1/project/{slug}/versions"), form)?;
        let wrapper: DataWrapper<ProjectVersion> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// Update an existing project version.
    ///
    /// `files` is an optional list of `(filename, bytes)` tuples to upload.
    pub fn update(
        &self,
        slug: &str,
        version: &str,
        params: &UpdateVersionParams,
        files: Option<&[(&str, Vec<u8>)]>,
    ) -> Result<ProjectVersion> {
        // The API requires `version` field in the body.
        let version_value = params.version_new.as_deref().unwrap_or(version);
        let mut form = multipart::Form::new().text("version", version_value.to_string());

        if let Some(ref v) = params.name {
            form = form.text("name", v.clone());
        }
        if let Some(ref v) = params.release_type {
            form = form.text("release_type", v.clone());
        }
        if let Some(ref v) = params.release_date {
            form = form.text("release_date", v.clone());
        }
        if let Some(ref v) = params.changelog {
            form = form.text("changelog", v.clone());
        }
        if params.clean_existing_files {
            form = form.text("clean_existing_files", "1");
        }

        if let Some(ref tags) = params.tags {
            for t in tags {
                form = form.text("tags[]", t.clone());
            }
        }

        if let Some(ref deps) = params.dependencies {
            for (i, dep) in deps.iter().enumerate() {
                form = form.text(format!("dependencies[{i}][project]"), dep.project.clone());
                form = form.text(format!("dependencies[{i}][version]"), dep.version.clone());
                form = form.text(format!("dependencies[{i}][type]"), dep.dep_type.clone());
                form = form.text(
                    format!("dependencies[{i}][external]"),
                    if dep.external { "1" } else { "0" }.to_string(),
                );
            }
        }

        if let Some(ref removals) = params.files_to_remove {
            for f in removals {
                form = form.text("files_to_remove[]", f.clone());
            }
        }

        if let Some(file_list) = files {
            for (filename, bytes) in file_list {
                let part = multipart::Part::bytes(bytes.clone())
                    .file_name(filename.to_string())
                    .mime_str("application/octet-stream")
                    .map_err(|e| HubApiError::Api {
                        status: 0,
                        message: format!("Invalid MIME type: {e}"),
                    })?;
                form = form.part("files[]", part);
            }
        }

        let data = self
            .base
            .post_multipart(&format!("/v1/project/{slug}/version/{version}"), form)?;
        let wrapper: DataWrapper<ProjectVersion> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// Delete a project version.
    pub fn delete(&self, slug: &str, version: &str) -> Result<()> {
        self.base
            .delete(&format!("/v1/project/{slug}/version/{version}"))?;
        Ok(())
    }
}

// ---- Tags -----------------------------------------------------------------

pub struct TagsClient<'a> {
    base: &'a BaseClient,
}

impl TagsClient<'_> {
    /// List all project tags.
    pub fn list_project_tags(
        &self,
        plain: bool,
        project_type: Option<&str>,
    ) -> Result<Vec<ProjectTag>> {
        let mut query: Vec<(String, String)> = vec![("plain".into(), plain.to_string())];
        if let Some(pt) = project_type {
            query.push(("project_type".into(), pt.into()));
        }
        let data = self.base.get("/v1/project_tags", &query)?;
        let wrapper: DataWrapper<Vec<ProjectTag>> =
            serde_json::from_value(data.unwrap_or_default()).map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// Get a single project tag by slug.
    pub fn get_project_tag(&self, slug: &str) -> Result<ProjectTag> {
        let data = self.base.get(&format!("/v1/project_tag/{slug}"), &[])?;
        let wrapper: DataWrapper<ProjectTag> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// List all version tags.
    pub fn list_version_tags(
        &self,
        plain: bool,
        project_type: Option<&str>,
    ) -> Result<Vec<ProjectVersionTag>> {
        let mut query: Vec<(String, String)> = vec![("plain".into(), plain.to_string())];
        if let Some(pt) = project_type {
            query.push(("project_type".into(), pt.into()));
        }
        let data = self.base.get("/v1/version_tags", &query)?;
        let wrapper: DataWrapper<Vec<ProjectVersionTag>> =
            serde_json::from_value(data.unwrap_or_default()).map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// Get a single version tag by slug.
    pub fn get_version_tag(&self, slug: &str) -> Result<ProjectVersionTag> {
        let data = self.base.get(&format!("/v1/version_tag/{slug}"), &[])?;
        let wrapper: DataWrapper<ProjectVersionTag> =
            serde_json::from_value(data.unwrap_or_default()).map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }
}

// ---- Users ----------------------------------------------------------------

pub struct UsersClient<'a> {
    base: &'a BaseClient,
}

impl UsersClient<'_> {
    /// Get a user profile by username.
    pub fn get(&self, name: &str) -> Result<User> {
        let data = self.base.get(&format!("/v1/user/{name}"), &[])?;
        let wrapper: DataWrapper<User> =
            serde_json::from_value(data.unwrap_or_default()).map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(wrapper.data)
    }

    /// Get projects owned by a user.
    pub fn get_projects(&self, name: &str) -> Result<PaginatedResponse<Project>> {
        let data = self.base.get(&format!("/v1/user/{name}/projects"), &[])?;
        let resp: PaginatedResponse<Project> = serde_json::from_value(data.unwrap_or_default())
            .map_err(|e| HubApiError::Api {
                status: 0,
                message: format!("Deserialization error: {e}"),
            })?;
        Ok(resp)
    }
}
