//! Hub01 Shop API client library for Rust.
//!
//! A Rust client library for interacting with the
//! [Hub01 Shop API](https://hub01-shop.srgnis.com/api). This library provides
//! an easy-to-use interface for managing projects, versions, tags, and users.
//!
//! # Quick Start
//!
//! ```no_run
//! use hub01_client::HubClient;
//!
//! let client = HubClient::new("https://hub01-shop.srgnis.com/api", None).unwrap();
//!
//! // List project types
//! let types = client.project_types().list().unwrap();
//! for t in &types {
//!     println!("{}: {}", t.name, t.slug);
//! }
//! ```

pub mod client;
pub mod error;
pub mod models;

// Re-export the main public types at the crate root for convenience.
pub use client::{
    CreateVersionParams, Dependency, HubClient, ListProjectsParams, ListVersionsParams,
    ProjectTypesClient, ProjectVersionsClient, ProjectsClient, TagsClient, UpdateVersionParams,
    UsersClient,
};
pub use error::HubApiError;
pub use models::{
    PaginatedResponse, Project, ProjectFile, ProjectTag, ProjectType, ProjectVersion,
    ProjectVersionDependency, ProjectVersionTag, User,
};
