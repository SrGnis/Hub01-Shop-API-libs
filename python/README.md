# Hub01 Shop Python Client

A Python client library for interacting with the [Hub01 Shop API](https://hub01-shop.srgnis.com/api). This library provides an easy-to-use interface for managing projects, versions, tags, and users.

## Features

- 🔍 Browse and search projects
- 📦 Manage project versions (create, update, delete)
- 🏷️ Work with project and version tags
- 👤 User profile and project management
- 🔐 API token authentication
- ✅ Full CRUD operations for project versions

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd hub01_shop_clients

# The client has minimal dependencies
pip install requests
```

## Quick Start

### Basic Usage (Read-Only)

```python
from hub01_client import HubClient

# Initialize client (no auth needed for read-only operations)
client = HubClient(base_url="https://hub01-shop.srgnis.com/api")

# List project types
types = client.project_types.list()
for t in types:
    print(f"{t.name}: {t.description}")

# Search for projects
projects = client.projects.list(
    search="minecraft",
    project_type="mod",
    per_page=10
)

for project in projects['data']:
    print(f"{project.name} - {project.downloads} downloads")

# Get project details
project = client.projects.get("my-project-slug")
print(f"{project.name}: {project.description}")

# List project versions
versions = client.versions.list("my-project-slug")
for version in versions['data']:
    print(f"Version {version.version} - {version.release_type}")
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

```python
from hub01_client import HubClient
from datetime import date

# Initialize with your API token
client = HubClient(
    base_url="https://hub01-shop.srgnis.com/api",
    token="your-api-token-here"
)

# Verify your token works
token_info = client.test_token()
print(f"Authenticated as: {token_info['user']['username']}")
```

## Complete Workflow Example

Here's a complete example showing how to create, update, and manage a project version:

```python
from hub01_client import HubClient
from datetime import date
import io

# Initialize authenticated client
client = HubClient(
    base_url="https://hub01-shop.srgnis.com/api",
    token="your-api-token-here"
)

# 1. Get your user's projects
user_projects = client.users.get_projects("your-username")
my_project = user_projects['data'][0]
print(f"Working with project: {my_project.name}")

# 2. Create a file to upload
with open("my-mod-v1.0.0.jar", "rb") as mod_file:
    # 3. Create a new version
    new_version = client.versions.create(
        slug=my_project.slug,
        name="Version 1.0.0 - Initial Release",
        version="1.0.0",
        release_type="release",  # release, rc, beta, or alpha
        release_date=date.today().isoformat(),
        changelog="- Initial release\n- Added cool features\n- Fixed bugs",
        files=[mod_file],
        tags=["forge", "fabric"],  # Optional version tags
        dependencies=[  # Optional dependencies
            {
                "project": "dependency-slug",
                "version": "2.0.0",
                "type": "required",  # required, optional, or embedded
                "external": False
            }
        ]
    )

print(f"Created version: {new_version.version}")
print(f"Downloads URL: {new_version.files[0].url}")

# 4. Update the version
updated = client.versions.update(
    slug=my_project.slug,
    version="1.0.0",
    name="Version 1.0.0 - Initial Release (Updated)",
    release_type="release",
    release_date=date.today().isoformat(),
    changelog="- Initial release\n- Added cool features\n- Fixed bugs\n- Updated description"
)

print(f"Updated version: {updated.version}")

# 5. List all versions of the project
versions = client.versions.list(my_project.slug, order_by="release_date")
for v in versions['data']:
    print(f"- {v.version} ({v.release_type}): {v.downloads} downloads")

# 6. Delete a version (if needed)
client.versions.delete(my_project.slug, "1.0.0")
```

## Advanced Usage

### Filtering and Pagination

```python
# Advanced project search
projects = client.projects.list(
    project_type="mod",
    search="magic",
    tags=["adventure", "magic"],
    version_tags=["forge"],
    order_by="downloads",  # name, created_at, latest_version, downloads
    order_direction="desc",  # asc or desc
    per_page=25,  # 10, 25, 50, or 100
    page=1
)

print(f"Total results: {projects['meta']['total']}")
print(f"Current page: {projects['meta']['current_page']}")

# Filter versions by tags
versions = client.versions.list(
    "my-project",
    tags=["forge", "1.20"],
    order_by="release_date",
    order_direction="desc"
)
```

### Working with Tags

```python
# List all project tags
tags = client.tags.list_project_tags(plain=True)
for tag in tags:
    print(f"{tag.name} ({tag.slug})")

# Get specific tag details
tag = client.tags.get_project_tag("technology")
print(f"{tag.name}: {tag.icon}")

# List version tags
version_tags = client.tags.list_version_tags(plain=True)
```

### Error Handling

```python
from hub01_client import (
    HubClient,
    HubAPIException,
    AuthenticationException,
    NotFoundException,
    ValidationException
)

try:
    client = HubClient(base_url="...", token="invalid-token")
    client.test_token()
except AuthenticationException as e:
    print(f"Authentication failed: {e}")
except ValidationException as e:
    print(f"Validation error: {e.message}")
    print(f"Errors: {e.errors}")
except NotFoundException as e:
    print(f"Resource not found: {e}")
except HubAPIException as e:
    print(f"API error: {e}")
```

## Running Tests

The integration test suite validates all client functionality:

```bash
# Run all tests (including authenticated tests)
python test_integration.py --username your-username --token your-token

# Or use credential files
echo "your-username" > username
echo "your-token" > api_key
python test_integration.py

# Run read-only tests without authentication
python test_integration.py --base-url https://hub01-shop.srgnis.com/api
```

The test suite covers:
1. Project types listing
2. Project tags listing
3. Version tags listing
4. Project listing and filtering
5. Project search
6. Version listing and filtering
7. Version details
8. Token validation (requires auth)
9. User profile retrieval (requires auth)
10. User projects listing (requires auth)
11. **Version creation** (requires auth)
12. **Version update** (requires auth)
13. **Version deletion** (requires auth)

## API Reference

For detailed API documentation, see the [OpenAPI specification](api.json) or visit the [API documentation](https://hub01-shop.srgnis.com/api/documentation).

## Project Structure

```
hub01_shop_clients/
├── hub01_client/           # Main client library
│   ├── __init__.py        # Package exports
│   ├── client.py          # Client classes
│   ├── models.py          # Data models
│   └── exceptions.py      # Custom exceptions
├── test_integration.py    # Integration test suite
├── usage_example.py       # Usage examples
├── api.json              # OpenAPI specification
└── README.md             # This file
```

## Important Notes

### Version Updates
When updating a project version, the API requires **all** of these fields even if you're only changing one:
- `name`
- `version` (can be the same as current version)
- `release_type`
- `release_date`

### Dependencies Format
Dependencies must be provided as a list of dictionaries with the following structure:
```python
{
    "project": "project-slug",      # Required
    "version": "1.0.0",            # Optional
    "type": "required",            # Required: required, optional, or embedded
    "external": False              # Required: True for external, False for platform projects
}
```

### Pagination
The `per_page` parameter only accepts: **10, 25, 50, or 100**. Other values will result in a validation error.

## License

[MIT](LICENSE.md)
