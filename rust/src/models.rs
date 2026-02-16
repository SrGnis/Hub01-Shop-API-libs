use serde::Deserialize;

// ---------------------------------------------------------------------------
// Generic paginated response wrapper
// ---------------------------------------------------------------------------

/// Wraps a paginated API response.  The `data` field holds the deserialized
/// items while `meta` and `links` carry pagination metadata exactly as returned
/// by the API.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
    #[serde(default)]
    pub links: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Project types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectType {
    pub name: String,
    pub slug: String,
    pub icon: String,
}

// ---------------------------------------------------------------------------
// Tags (used for both project tags and version tags)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectTag {
    pub name: String,
    pub slug: String,
    pub icon: String,
    pub tag_group: String,
    pub project_types: Vec<String>,
    pub main_tag: String,
    #[serde(default)]
    pub sub_tags: Vec<ProjectTag>,
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub description: Option<String>,
    pub logo_url: String,
    pub website: Option<String>,
    pub issues: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub downloads: u64,
    pub created_at: String,
    pub last_release_date: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub version_count: u64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub members: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Project files
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectFile {
    pub name: String,
    pub size: u64,
    pub sha1: String,
    pub url: String,
}

// ---------------------------------------------------------------------------
// Dependencies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectVersionDependency {
    #[serde(rename = "project")]
    pub project_slug: String,
    #[serde(rename = "version")]
    pub version_slug: String,
    #[serde(rename = "type")]
    pub dep_type: String,
    pub external: bool,
}

// ---------------------------------------------------------------------------
// Project versions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectVersion {
    pub name: String,
    pub version: String,
    pub release_type: String,
    pub release_date: String,
    pub changelog: Option<String>,
    pub downloads: u64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub files: Vec<ProjectFile>,
    #[serde(default)]
    pub dependencies: Vec<ProjectVersionDependency>,
}

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub username: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub created_at: String,
}
