#!/usr/bin/env python3
"""
Hub01 Shop API Integration Test Suite

Tests all major endpoints in the following order:
1. Project types
2. Project tags
3. Version tags
4. List projects
5. Filter projects
6. List versions of one project
7. Filter versions of one project
8. Token test (if authenticated)
9. Get user (if authenticated)
10. Get user projects (if authenticated)
11. Create a project version (if authenticated)
12. Update a project version (if authenticated)
13. Delete a project version (if authenticated)

Usage:
    python test_integration.py [--username USERNAME] [--token TOKEN]
    
If credentials are not provided via CLI, the script will attempt to read from:
    - ./username (for username)
    - ./api_key (for token)
"""

import sys
import os
import argparse
import io
import json
from datetime import date

# Ensure the current directory is in the path
sys.path.insert(0, os.path.abspath('.'))

from hub01_client import HubClient, HubAPIException, AuthenticationException

def read_credential_file(filename):
    """Read credential from file if exists"""
    try:
        if os.path.exists(filename):
            with open(filename, 'r') as f:
                return f.read().strip()
    except:
        pass
    return None

def create_dummy_file(content="Test file content"):
    """Create a dummy file for upload testing"""
    return io.BytesIO(content.encode('utf-8'))

def run_tests(base_url, username=None, token=None):
    print(f"=== Hub01 Shop API Integration Test ===")
    print(f"Base URL: {base_url}")
    print(f"Username: {username or 'Not provided'}")
    print(f"Token: {'Present' if token else 'Not provided'}")
    print(f"Auth Tests: {'Enabled' if (username and token) else 'Disabled'}")
    print("=" * 50)
    
    # Initialize client (without auth for read-only tests)
    client = HubClient(base_url=base_url)
    
    # Will be populated during tests
    test_project_slug = None
    test_version_slug = None
    created_version_slug = None
    
    # ========================================
    # 1. TEST PROJECT TYPES
    # ========================================
    print("\n[1/13] Testing: List Project Types")
    try:
        types = client.project_types.list()
        print(f"✓ Found {len(types)} project types")
        for t in types[:3]:
            print(f"  - {t.name} ({t.slug})")
    except Exception as e:
        print(f"✗ FAILED: {e}")
        return
    
    # ========================================
    # 2. TEST PROJECT TAGS
    # ========================================
    print("\n[2/13] Testing: List Project Tags")
    try:
        tags = client.tags.list_project_tags(plain=True)
        print(f"✓ Found {len(tags)} project tags")
        if tags:
            print(f"  - First tag: {tags[0].name} ({tags[0].slug})")
            
            # Get specific tag
            tag_detail = client.tags.get_project_tag(tags[0].slug)
            print(f"  - Tag detail icon: {tag_detail.icon}")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # 3. TEST VERSION TAGS
    # ========================================
    print("\n[3/13] Testing: List Version Tags")
    try:
        version_tags = client.tags.list_version_tags(plain=True)
        print(f"✓ Found {len(version_tags)} version tags")
        if version_tags:
            print(f"  - First tag: {version_tags[0].name}")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # 4. TEST LIST PROJECTS
    # ========================================
    print("\n[4/13] Testing: List Projects")
    try:
        projects_resp = client.projects.list(project_type='mod', per_page=10)
        projects = projects_resp.get('data', [])
        print(f"✓ Found {len(projects)} projects (page 1)")
        for p in projects[:3]:
            print(f"  - {p.name} (downloads: {p.downloads})")
        
        if projects:
            test_project_slug = projects[0].slug
            print(f"  → Using '{test_project_slug}' for subsequent tests")
    except Exception as e:
        print(f"✗ FAILED: {e}")
        return
    
    # ========================================
    # 5. TEST FILTER PROJECTS
    # ========================================
    print("\n[5/13] Testing: Filter Projects (search)")
    try:
        if test_project_slug and projects:
            # Search for the first project by name
            search_term = projects[0].name.split()[0]
            search_results = client.projects.list(search=search_term, per_page=10)
            print(f"✓ Search for '{search_term}' returned {search_results['meta']['total']} results")
            
            # Test filtering by project type
            filtered = client.projects.list(project_type='mod', order_by='name', order_direction='asc', per_page=10)
            print(f"✓ Filtered by type 'mod', ordered by name: {len(filtered['data'])} results")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # 6. TEST LIST VERSIONS OF ONE PROJECT
    # ========================================
    print("\n[6/13] Testing: List Versions of Project")
    try:
        if test_project_slug:
            versions_resp = client.versions.list(test_project_slug, per_page=10)
            versions = versions_resp.get('data', [])
            print(f"✓ Project '{test_project_slug}' has {len(versions)} versions")
            
            if versions:
                test_version_slug = versions[0].version
                for v in versions[:3]:
                    print(f"  - {v.version} ({v.release_type}, downloads: {v.downloads})")
                print(f"  → Using version '{test_version_slug}' for subsequent tests")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # 7. TEST FILTER VERSIONS OF ONE PROJECT
    # ========================================
    print("\n[7/13] Testing: Filter Versions")
    try:
        if test_project_slug:
            # Filter by release type using tags would work, ordering by date
            filtered_versions = client.versions.list(
                test_project_slug, 
                order_by='release_date',
                order_direction='desc',
                per_page=10
            )
            print(f"✓ Filtered versions (by release_date desc): {len(filtered_versions['data'])} results")
            
            # Get specific version detail
            if test_version_slug:
                version_detail = client.versions.get(test_project_slug, test_version_slug)
                print(f"✓ Got version details: {version_detail.version}")
                print(f"  - Files: {len(version_detail.files)}")
                print(f"  - Dependencies: {len(version_detail.dependencies)}")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # AUTHENTICATED TESTS
    # ========================================
    if not (username and token):
        print("\n" + "=" * 50)
        print("⚠ Skipping authenticated tests (no credentials provided)")
        print("=" * 50)
        return
    
    # Re-initialize client with authentication
    print("\n" + "=" * 50)
    print("🔐 Authenticated Tests")
    print("=" * 50)
    
    auth_client = HubClient(base_url=base_url, token=token)
    
    # ========================================
    # 8. TEST TOKEN
    # ========================================
    print("\n[8/13] Testing: Token Validation")
    try:
        token_info = auth_client.test_token()
        print(f"✓ Token is valid")
        print(f"  - User: {token_info.get('user', {}).get('uername', 'N/A')}")
        print(f"  - Token name: {token_info.get('token', {}).get('name', 'N/A')}")
    except AuthenticationException as e:
        print(f"✗ FAILED: Invalid token - {e}")
        return
    except Exception as e:
        print(f"✗ FAILED: {e}")
        return
    
    # ========================================
    # 9. TEST GET USER
    # ========================================
    print("\n[9/13] Testing: Get User Profile")
    try:
        user = auth_client.users.get(username)
        print(f"✓ Retrieved user: {user.username}")
        print(f"  - Bio: {user.bio or 'None'}")
        print(f"  - Avatar: {'Present' if user.avatar else 'None'}")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # 10. TEST GET USER PROJECTS
    # ========================================
    print("\n[10/13] Testing: Get User Projects")
    try:
        user_projects = auth_client.users.get_projects(username)
        user_project_list = user_projects.get('data', [])
        print(f"✓ User has {len(user_project_list)} projects")
        
        # Use one of user's projects for create/update/delete tests
        if user_project_list:
            test_project_slug = user_project_list[0].slug
            print(f"  - Using user project: {test_project_slug}")
        else:
            print(f"  ⚠ User has no projects, cannot test create/update/delete")
            return
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # 11. TEST CREATE PROJECT VERSION
    # ========================================
    print("\n[11/13] Testing: Create Project Version")
    try:
        # Create a test version
        today = date.today().isoformat()
        created_version_slug = f"test-api-{date.today().strftime('%Y%m%d-%H%M%S')}"
        
        # Create a dummy file
        test_file = create_dummy_file("This is a test file created by integration test")
        test_file.name = "test_file.txt"
        
        # Get a random project to use as a dependency
        dependency_project = None
        dependency_version = None
        
        try:
            # Get list of projects (excluding the one we're creating a version for)
            dep_projects = auth_client.projects.list(per_page=10)['data']
            for proj in dep_projects:
                if proj.slug != test_project_slug:
                    # Get versions of this project
                    dep_versions = auth_client.versions.list(proj.slug, per_page=10)['data']
                    if dep_versions:
                        dependency_project = proj.slug
                        dependency_version = dep_versions[0].version
                        print(f"  Using dependency: {dependency_project} v{dependency_version}")
                        break
        except:
            pass
        
        # Build dependencies list if we found a valid one
        dependencies_list = None
        if dependency_project and dependency_version:
            dependencies_list = [
                {
                    "project": dependency_project,
                    "version": dependency_version,
                    "type": "optional",
                    "external": False
                }
            ]
        
        # Get version tags to add to the version
        version_tags_list = []
        try:
            # Get first 5 version tags
            all_version_tags = auth_client.tags.list_version_tags(plain=True)
            if all_version_tags and len(all_version_tags) >= 2:
                version_tags_list = [all_version_tags[0].slug, all_version_tags[1].slug]
                print(f"  Using version tags: {', '.join(version_tags_list)}")
        except:
            pass
        
        new_version = auth_client.versions.create(
            slug=test_project_slug,
            name=f"Test Version {created_version_slug}",
            version=created_version_slug,
            release_type="alpha",
            release_date=today,
            changelog="Test version created by integration test",
            files=[test_file],
            dependencies=dependencies_list,
            tags=version_tags_list if version_tags_list else None
        )
        
        print(f"✓ Created version: {new_version.version}")
        print(f"  - Name: {new_version.name}")
        print(f"  - Release type: {new_version.release_type}")
        print(f"  - Files: {len(new_version.files)}")
        print(f"  - Dependencies: {len(new_version.dependencies)}")
        print(f"  - Tags: {len(new_version.tags)}")
        
    except Exception as e:
        print(f"✗ FAILED: {e}")
        created_version_slug = None
    
    # ========================================
    # 12. TEST UPDATE PROJECT VERSION
    # ========================================
    print("\n[12/13] Testing: Update Project Version")
    try:
        if created_version_slug:
            # API requires name, version, release_type, and release_date even for updates
            today = date.today().isoformat()
            
            # Try to get a second project for an additional dependency
            update_dependencies = None
            try:
                dep_projects = auth_client.projects.list(per_page=25)['data']
                found_deps = []
                for proj in dep_projects:
                    if proj.slug != test_project_slug:
                        dep_versions = auth_client.versions.list(proj.slug, per_page=10)['data']
                        if dep_versions and len(found_deps) < 2:
                            found_deps.append({
                                "project": proj.slug,
                                "version": dep_versions[0].version,
                                "type": "optional" if len(found_deps) == 0 else "required",
                                "external": False
                            })
                        if len(found_deps) >= 2:
                            break
                
                if found_deps:
                    update_dependencies = found_deps
                    print(f"  Updating with {len(update_dependencies)} dependencies")
            except:
                pass
            
            # Get different version tags for the update
            update_tags_list = []
            try:
                all_version_tags = auth_client.tags.list_version_tags(plain=True)
                # Use different tags than creation (skip first 2, use next 3)
                if all_version_tags and len(all_version_tags) >= 5:
                    update_tags_list = [all_version_tags[2].slug, all_version_tags[3].slug, all_version_tags[4].slug]
                    print(f"  Updating with version tags: {', '.join(update_tags_list)}")
            except:
                pass
            
            updated_version = auth_client.versions.update(
                slug=test_project_slug,
                version=created_version_slug,
                name=f"Updated Test Version {created_version_slug}",
                release_type="beta",
                release_date=today,
                changelog="Updated changelog via integration test",
                dependencies=update_dependencies,
                tags=update_tags_list if update_tags_list else None
            )
            
            print(f"✓ Updated version: {updated_version.version}")
            print(f"  - New name: {updated_version.name}")
            print(f"  - New release type: {updated_version.release_type}")
            print(f"  - Dependencies: {len(updated_version.dependencies)}")
            print(f"  - Tags: {len(updated_version.tags)}")
            print(f"  - Changelog: {updated_version.changelog[:50]}...")
        else:
            print(f"  ⚠ Skipping (no version was created)")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    # ========================================
    # 13. TEST DELETE PROJECT VERSION
    # ========================================
    print("\n[13/13] Testing: Delete Project Version")
    try:
        if created_version_slug:
            auth_client.versions.delete(test_project_slug, created_version_slug)
            print(f"✓ Deleted version: {created_version_slug}")
            
            # Verify deletion
            try:
                auth_client.versions.get(test_project_slug, created_version_slug)
                print(f"  ⚠ Warning: Version still exists after deletion")
            except:
                print(f"  ✓ Verified: Version no longer exists")
        else:
            print(f"  ⚠ Skipping (no version was created)")
    except Exception as e:
        print(f"✗ FAILED: {e}")
    
    print("\n" + "=" * 50)
    print("✓ All tests completed!")
    print("=" * 50)

def main():
    parser = argparse.ArgumentParser(description='Hub01 Shop API Integration Tests')
    parser.add_argument('--username', help='Username for authenticated tests')
    parser.add_argument('--token', help='API token for authenticated tests')
    parser.add_argument('--base-url', default='http://127.0.0.1:8000/api', 
                       help='API base URL (default: http://127.0.0.1:8000/api)')
    
    args = parser.parse_args()
    
    # Try to get credentials from args or files
    username = args.username or read_credential_file('username')
    token = args.token or read_credential_file('api_key')
    
    run_tests(args.base_url, username, token)

if __name__ == "__main__":
    main()
