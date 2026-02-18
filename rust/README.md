# Hub01 Shop Rust Client

A Rust client library for interacting with the [Hub01 Shop API](https://hub01-shop.srgnis.com/api). This library provides an easy-to-use interface for managing projects, versions, tags, and users.

## Features

- 🔍 Browse and search projects
- 📦 Manage project versions (create, update, delete)
- 🏷️ Work with project and version tags
- 👤 User profile and project management
- 🔐 API token authentication
- ✅ Full CRUD operations for project versions

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hub01-client = "0.1.2"
```

## Quick Start

### Basic Usage (Read-Only)

```rust
use hub01_client::{HubClient, ListProjectsParams, ListVersionsParams};

fn main() -> hub01_client::error::Result<()> {
    // Initialize client (no auth needed for read-only operations)
    let client = HubClient::new("https://hub01-shop.srgnis.com/api", None)?;

    // List project types
    let types = client.project_types().list()?;
    for t in &types {
        println!("{}: {}", t.name, t.slug);
    }

    // Search for projects
    let projects = client.projects().list(&ListProjectsParams {
        search: Some("minecraft".into()),
        project_type: Some("mod".into()),
        per_page: 10,
        ..Default::default()
    })?;

    for project in &projects.data {
        println!("{} - {} downloads", project.name, project.downloads);
    }

    // Get project details
    let project = client.projects().get("my-project-slug")?;
    println!("{}: {}", project.name, project.summary);

    // List project versions
    let versions = client.versions().list("my-project-slug", &ListVersionsParams::default())?;
    for version in &versions.data {
        println!("Version {} - {}", version.version, version.release_type);
    }

    Ok(())
}
```

### Authenticated Operations

To create, update, or delete project versions, you need an API token.

#### Getting Your API Token

1. Log in to [Hub01 Shop](https://hub01-shop.srgnis.com)
2. Go to your user profile
3. Navigate to **API Tokens**
4. Click **Create API Token**
5. Provide a name and optional expiration date
6. Copy the generated token

#### Using Authentication

```rust
use hub01_client::HubClient;

fn main() -> hub01_client::error::Result<()> {
    // Initialize with your API token
    let client = HubClient::new(
        "https://hub01-shop.srgnis.com/api",
        Some("your-api-token-here"),
    )?;

    // Verify your token works
    let token_info = client.test_token()?;
    println!("Authenticated as: {}", token_info["user"]["username"]);

    Ok(())
}
```

## Complete Workflow Example

```rust
use hub01_client::{HubClient, CreateVersionParams, UpdateVersionParams, Dependency};
use std::fs;

fn main() -> hub01_client::error::Result<()> {
    let client = HubClient::new(
        "https://hub01-shop.srgnis.com/api",
        Some("your-api-token-here"),
    )?;

    // 1. Get your user's projects
    let user_projects = client.users().get_projects("your-username")?;
    let my_project = &user_projects.data[0];
    println!("Working with project: {}", my_project.name);

    // 2. Read a file to upload
    let file_bytes = fs::read("my-mod-v1.0.0.jar").unwrap();

    // 3. Create a new version
    let new_version = client.versions().create(
        &my_project.slug,
        &CreateVersionParams {
            name: "Version 1.0.0 - Initial Release".into(),
            version: "1.0.0".into(),
            release_type: "release".into(),
            release_date: "2025-01-01".into(),
            changelog: "- Initial release\n- Added cool features".into(),
            tags: Some(vec!["forge".into(), "fabric".into()]),
            dependencies: Some(vec![Dependency {
                project: "dependency-slug".into(),
                version: "2.0.0".into(),
                dep_type: "required".into(),
                external: false,
            }]),
        },
        &[("my-mod-v1.0.0.jar", file_bytes)],
    )?;

    println!("Created version: {}", new_version.version);

    // 4. Update the version
    let updated = client.versions().update(
        &my_project.slug,
        "1.0.0",
        &UpdateVersionParams {
            name: Some("Version 1.0.0 - Initial Release (Updated)".into()),
            release_type: Some("release".into()),
            release_date: Some("2025-01-01".into()),
            changelog: Some("- Updated description".into()),
            ..Default::default()
        },
        None,
    )?;
    println!("Updated version: {}", updated.version);

    // 5. Delete a version
    client.versions().delete(&my_project.slug, "1.0.0")?;

    Ok(())
}
```

## Advanced Usage

### Filtering and Pagination

```rust
use hub01_client::{HubClient, ListProjectsParams, ListVersionsParams};

let client = HubClient::new("https://hub01-shop.srgnis.com/api", None).unwrap();

// Advanced project search
let projects = client.projects().list(&ListProjectsParams {
    project_type: Some("mod".into()),
    search: Some("magic".into()),
    tags: Some(vec!["adventure".into(), "magic".into()]),
    version_tags: Some(vec!["forge".into()]),
    order_by: Some("downloads".into()),
    order_direction: Some("desc".into()),
    per_page: 25,
    page: 1,
    ..Default::default()
}).unwrap();

// Filter versions by tags
let versions = client.versions().list("my-project", &ListVersionsParams {
    tags: Some(vec!["forge".into(), "1.20".into()]),
    order_by: "release_date".into(),
    order_direction: "desc".into(),
    ..Default::default()
}).unwrap();
```

### Working with Tags

```rust
use hub01_client::HubClient;

let client = HubClient::new("https://hub01-shop.srgnis.com/api", None).unwrap();

// List all project tags
let tags = client.tags().list_project_tags(true, None).unwrap();
for tag in &tags {
    println!("{} ({})", tag.name, tag.slug);
}

// Get specific tag details
let tag = client.tags().get_project_tag("technology").unwrap();
println!("{}: {}", tag.name, tag.icon);

// List version tags
let version_tags = client.tags().list_version_tags(true, None).unwrap();
```

### Error Handling

```rust
use hub01_client::{HubClient, HubApiError};

let client = HubClient::new("https://hub01-shop.srgnis.com/api", Some("invalid-token")).unwrap();

match client.test_token() {
    Ok(info) => println!("Authenticated!"),
    Err(HubApiError::Authentication { message }) => {
        println!("Authentication failed: {message}");
    }
    Err(HubApiError::Validation { message, errors }) => {
        println!("Validation error: {message}");
        if let Some(e) = errors {
            println!("Errors: {e}");
        }
    }
    Err(HubApiError::NotFound { message }) => {
        println!("Not found: {message}");
    }
    Err(e) => println!("Other error: {e}"),
}
```

## Running Tests

```bash
# Run read-only tests
cargo test -- --nocapture

# Run with authentication (enables create/update/delete tests)
HUB01_USERNAME=your-username HUB01_TOKEN=your-token cargo test -- --nocapture

# Or use credential files
echo "your-username" > username
echo "your-token" > api_key
cargo test -- --nocapture
```

## Project Structure

```
rust/
├── Cargo.toml              # Package manifest
├── README.md               # This file
├── src/
│   ├── lib.rs              # Crate root & re-exports
│   ├── client.rs           # HubClient + sub-clients
│   ├── error.rs            # Error types
│   └── models.rs           # Data models
└── tests/
    └── integration.rs      # Integration test suite
```

## Important Notes

### Version Updates
When updating a project version, the API requires **all** of these fields even if you're only changing one:
- `name`
- `version` (can be the same as current version)
- `release_type`
- `release_date`

### Dependencies Format
Dependencies are specified as `Dependency` structs:
```rust
Dependency {
    project: "project-slug".into(),  // Required
    version: "1.0.0".into(),         // Optional
    dep_type: "required".into(),     // Required: required, optional, or embedded
    external: false,                 // Required: true for external, false for platform
}
```

### Pagination
The `per_page` parameter only accepts: **10, 25, 50, or 100**. Other values will result in a validation error.

## License

[MIT](LICENSE.md)
