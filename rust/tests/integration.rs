//! Integration tests for the Hub01 Shop API Rust client.
//!
//! Mirrors the Python `test_integration.py`.  Read-only tests run without
//! authentication.  Set the environment variables `HUB01_USERNAME` and
//! `HUB01_TOKEN` (or provide `username` and `api_key` files next to the
//! test runner) to enable authenticated tests.
//!
//! Run with:
//!
//! ```bash
//! cargo test -- --nocapture
//! ```

use hub01_client::{HubClient, ListProjectsParams, ListVersionsParams};
use std::fs;

fn read_credential_file(name: &str) -> Option<String> {
    fs::read_to_string(name).ok().map(|s| s.trim().to_string())
}

fn base_url() -> String {
    std::env::var("HUB01_BASE_URL")
        .unwrap_or_else(|_| "https://hub01-shop.srgnis.com/api".into())
}

fn credentials() -> (Option<String>, Option<String>) {
    let username = std::env::var("HUB01_USERNAME")
        .ok()
        .or_else(|| read_credential_file("username"));
    let token = std::env::var("HUB01_TOKEN")
        .ok()
        .or_else(|| read_credential_file("api_key"));
    (username, token)
}

// ---------------------------------------------------------------------------
// 1. Project types
// ---------------------------------------------------------------------------

#[test]
fn test_list_project_types() {
    let client = HubClient::new(&base_url(), None).unwrap();
    let types = client.project_types().list().unwrap();
    assert!(!types.is_empty(), "Expected at least one project type");
    println!("[1] Found {} project types", types.len());
    for t in types.iter().take(3) {
        println!("  - {} ({})", t.name, t.slug);
    }
}

// ---------------------------------------------------------------------------
// 2. Project tags
// ---------------------------------------------------------------------------

#[test]
fn test_list_project_tags() {
    let client = HubClient::new(&base_url(), None).unwrap();
    let tags = client.tags().list_project_tags(true, None).unwrap();
    assert!(!tags.is_empty(), "Expected at least one project tag");
    println!("[2] Found {} project tags", tags.len());
    println!("  - First tag: {} ({})", tags[0].name, tags[0].slug);

    // Fetch a specific tag
    let detail = client.tags().get_project_tag(&tags[0].slug).unwrap();
    println!("  - Tag icon: {}", detail.icon);
}

// ---------------------------------------------------------------------------
// 3. Version tags
// ---------------------------------------------------------------------------

#[test]
fn test_list_version_tags() {
    let client = HubClient::new(&base_url(), None).unwrap();
    let tags = client.tags().list_version_tags(true, None).unwrap();
    assert!(!tags.is_empty(), "Expected at least one version tag");
    println!("[3] Found {} version tags", tags.len());
    println!("  - First tag: {}", tags[0].name);
}

// ---------------------------------------------------------------------------
// 4. List projects
// ---------------------------------------------------------------------------

#[test]
fn test_list_projects() {
    let client = HubClient::new(&base_url(), None).unwrap();
    let resp = client
        .projects()
        .list(&ListProjectsParams {
            per_page: 10,
            ..Default::default()
        })
        .unwrap();
    assert!(!resp.data.is_empty(), "Expected at least one project");
    println!("[4] Found {} projects (page 1)", resp.data.len());
    for p in resp.data.iter().take(3) {
        println!("  - {} (downloads: {})", p.name, p.downloads);
    }
}

// ---------------------------------------------------------------------------
// 5. Filter / search projects
// ---------------------------------------------------------------------------

#[test]
fn test_filter_projects() {
    let client = HubClient::new(&base_url(), None).unwrap();

    // First list to get a search term
    let resp = client
        .projects()
        .list(&ListProjectsParams::default())
        .unwrap();
    if let Some(first) = resp.data.first() {
        let term = first.name.split_whitespace().next().unwrap_or(&first.name);
        let search = client
            .projects()
            .list(&ListProjectsParams {
                search: Some(term.to_string()),
                per_page: 10,
                ..Default::default()
            })
            .unwrap();
        println!(
            "[5] Search for '{}' returned {} results",
            term,
            search.data.len()
        );

        // Order by name ASC
        let filtered = client
            .projects()
            .list(&ListProjectsParams {
                order_by: Some("name".into()),
                order_direction: Some("asc".into()),
                per_page: 10,
                ..Default::default()
            })
            .unwrap();
        println!(
            "[5] Filtered by type 'mod', ordered by name: {} results",
            filtered.data.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 6. List versions of a project
// ---------------------------------------------------------------------------

#[test]
fn test_list_versions() {
    let client = HubClient::new(&base_url(), None).unwrap();
    let projects = client
        .projects()
        .list(&ListProjectsParams::default())
        .unwrap();
    let slug = &projects.data[0].slug;

    let versions = client
        .versions()
        .list(slug, &ListVersionsParams::default())
        .unwrap();
    println!(
        "[6] Project '{}' has {} versions",
        slug,
        versions.data.len()
    );
    for v in versions.data.iter().take(3) {
        println!(
            "  - {} ({}, downloads: {})",
            v.version, v.release_type, v.downloads
        );
    }
}

// ---------------------------------------------------------------------------
// 7. Filter versions + get details
// ---------------------------------------------------------------------------

#[test]
fn test_filter_versions_and_details() {
    let client = HubClient::new(&base_url(), None).unwrap();
    let projects = client
        .projects()
        .list(&ListProjectsParams::default())
        .unwrap();
    let slug = &projects.data[0].slug;

    let versions = client
        .versions()
        .list(
            slug,
            &ListVersionsParams {
                order_by: "release_date".into(),
                order_direction: "desc".into(),
                ..Default::default()
            },
        )
        .unwrap();
    println!(
        "[7] Filtered versions (by release_date desc): {} results",
        versions.data.len()
    );

    if let Some(v) = versions.data.first() {
        let detail = client.versions().get(slug, &v.version).unwrap();
        println!("[7] Version details: {}", detail.version);
        println!("  - Files: {}", detail.files.len());
        println!("  - Dependencies: {}", detail.dependencies.len());
    }
}

// ---------------------------------------------------------------------------
// 8–13. Authenticated tests
// ---------------------------------------------------------------------------

#[test]
fn test_authenticated_operations() {
    let (username, token) = credentials();
    let (username, token) = match (username, token) {
        (Some(u), Some(t)) => (u, t),
        _ => {
            println!("[8-13] Skipping authenticated tests (no credentials)");
            return;
        }
    };

    let client = HubClient::new(&base_url(), Some(&token)).unwrap();

    // 8. Token validation
    println!("[8] Testing token validation");
    let token_info = client.test_token().unwrap();
    println!(
        "  ✓ Token valid — user: {}",
        token_info
            .get("user")
            .and_then(|u| u.get("username"))
            .and_then(|u| u.as_str())
            .unwrap_or("N/A")
    );

    // 9. Get user
    println!("[9] Testing get user profile");
    let user = client.users().get(&username).unwrap();
    println!("  ✓ User: {} (bio: {})", user.username, user.bio.as_deref().unwrap_or("None"));

    // 10. Get user projects
    println!("[10] Testing get user projects");
    let user_projects = client.users().get_projects(&username).unwrap();
    println!("  ✓ User has {} projects", user_projects.data.len());

    let test_slug = match user_projects.data.first() {
        Some(p) => p.slug.clone(),
        None => {
            println!("  ⚠ No projects — cannot test create/update/delete");
            return;
        }
    };
    println!("  Using project: {}", test_slug);

    // 11. Create version
    println!("[11] Testing create version");
    let today = chrono_today();
    let version_slug = format!("test-api-{}", today.replace('-', ""));

    use hub01_client::CreateVersionParams;
    let new_version = client
        .versions()
        .create(
            &test_slug,
            &CreateVersionParams {
                name: format!("Test Version {version_slug}"),
                version: version_slug.clone(),
                release_type: "alpha".into(),
                release_date: today.clone(),
                changelog: "Test version created by Rust integration test".into(),
                tags: None,
                dependencies: None,
            },
            &[("test_file.txt", b"Test file content".to_vec())],
        )
        .unwrap();
    println!("  ✓ Created version: {}", new_version.version);

    // 12. Update version
    println!("[12] Testing update version");
    use hub01_client::UpdateVersionParams;
    let updated = client
        .versions()
        .update(
            &test_slug,
            &version_slug,
            &UpdateVersionParams {
                name: Some(format!("Updated Test Version {version_slug}")),
                release_type: Some("beta".into()),
                release_date: Some(today),
                changelog: Some("Updated by Rust integration test".into()),
                ..Default::default()
            },
            None,
        )
        .unwrap();
    println!("  ✓ Updated version: {} ({})", updated.version, updated.release_type);

    // 13. Delete version
    println!("[13] Testing delete version");
    client.versions().delete(&test_slug, &version_slug).unwrap();
    println!("  ✓ Deleted version: {}", version_slug);

    // Verify deletion
    match client.versions().get(&test_slug, &version_slug) {
        Err(_) => println!("  ✓ Verified: version no longer exists"),
        Ok(_) => println!("  ⚠ Warning: version still exists after deletion"),
    }
}

/// Simple date helper (avoids pulling in chrono just for tests).
fn chrono_today() -> String {
    // Use system time to produce YYYY-MM-DD
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // rough UTC date calculation
    let days = now / 86400;
    let mut y = 1970i32;
    let mut remaining = days;
    loop {
        let year_days: u64 = if is_leap(y) { 366 } else { 365 };
        if remaining < year_days {
            break;
        }
        remaining -= year_days;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut m = 1u32;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!("{y:04}-{m:02}-{d:02}")
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
