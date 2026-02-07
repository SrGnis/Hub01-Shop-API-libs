import requests
from typing import List, Optional, Dict, Any, Union
from .exceptions import (
    HubAPIException, AuthenticationException, PermissionDeniedException,
    NotFoundException, ValidationException
)
from .models import (
    ProjectType, Project, ProjectVersion, ProjectTag, User
)

class BaseClient:
    def __init__(self, base_url: str, token: Optional[str] = None):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        if token:
            self.session.headers.update({'Authorization': f'Bearer {token}'})
        self.session.headers.update({'Accept': 'application/json'})

    def _request(self, method: str, endpoint: str, **kwargs) -> Any:
        url = f"{self.base_url}{endpoint}"
        try:
            response = self.session.request(method, url, **kwargs)
        except requests.RequestException as e:
            raise HubAPIException(f"Request failed: {str(e)}") from e

        if response.status_code == 204:
            return None

        try:
            data = response.json()
        except ValueError:
            if 200 <= response.status_code < 300:
                return response.content
            data = {}

        if 200 <= response.status_code < 300:
            return data

        if response.status_code == 401:
            raise AuthenticationException(data.get('message', 'Unauthenticated'))
        if response.status_code == 403:
            # Handle nested message in anyOf schema wrapper if present, otherwise direct message
            msg = data.get('message', 'Permission denied')
            if isinstance(data, dict) and 'message' not in data: 
                 # Sometimes error wrapping is complex, keeping simple for now based on spec
                 pass
            raise PermissionDeniedException(msg)
        if response.status_code == 404:
            raise NotFoundException(data.get('message', 'Not found'))
        if response.status_code == 422:
            raise ValidationException(data.get('message', 'Validation error'), data.get('errors'))
        
        raise HubAPIException(f"API Error {response.status_code}: {data.get('message', response.text)}")

class ProjectTypesClient(BaseClient):
    def list(self) -> List[ProjectType]:
        """List all project types"""
        response = self._request('GET', '/v1/project_types')
        return [ProjectType.from_dict(item) for item in response.get('data', [])]

    def get(self, slug: str) -> ProjectType:
        """Get a project type by slug"""
        response = self._request('GET', f'/v1/project_type/{slug}')
        return ProjectType.from_dict(response.get('data'))

class ProjectsClient(BaseClient):
    def list(self, 
             project_type: Optional[str] = 'mod',
             search: Optional[str] = None,
             tags: Optional[List[str]] = None,
             version_tags: Optional[List[str]] = None,
             order_by: Optional[str] = 'downloads',
             order_direction: Optional[str] = 'desc',
             per_page: int = 10,
             page: int = 1,
             release_date_period: str = 'all',
             release_date_start: Optional[str] = None,
             release_date_end: Optional[str] = None) -> Dict[str, Any]:
        """
        Search projects. Returns a paginated response with data and meta.
        Use .get('data') to get list of Project objects manually or helpers.
        """
        params = {
            'project_type': project_type,
            'search': search,
            'tags[]': tags,
            'version_tags[]': version_tags,
            'order_by': order_by,
            'order_direction': order_direction,
            'per_page': per_page,
            'page': page,
            'release_date_period': release_date_period,
            'release_date_start': release_date_start,
            'release_date_end': release_date_end
        }
        # Remove None values
        params = {k: v for k, v in params.items() if v is not None}
        
        response = self._request('GET', '/v1/projects', params=params)
        
        # Convert data items to objects
        projects = [Project.from_dict(item) for item in response.get('data', [])]
        response['data'] = projects
        return response

    def get(self, slug: str) -> Project:
        """Get a project by slug"""
        response = self._request('GET', f'/v1/project/{slug}')
        return Project.from_dict(response.get('data'))

class ProjectVersionsClient(BaseClient):
    def list(self, 
             slug: str,
             tags: Optional[List[str]] = None,
             order_by: str = 'downloads',
             order_direction: str = 'desc',
             per_page: int = 10,
             page: int = 1) -> Dict[str, Any]:
        """List all versions of a project"""
        params = {
            'tags[]': tags,
            'order_by': order_by,
            'order_direction': order_direction,
            'per_page': per_page,
            'page': page
        }
        params = {k: v for k, v in params.items() if v is not None}
        
        response = self._request('GET', f'/v1/project/{slug}/versions', params=params)
        versions = [ProjectVersion.from_dict(item) for item in response.get('data', [])]
        response['data'] = versions
        return response

    def get(self, slug: str, version: str) -> ProjectVersion:
        """Get a project version"""
        response = self._request('GET', f'/v1/project/{slug}/version/{version}')
        return ProjectVersion.from_dict(response.get('data'))

    def create(self, slug: str, 
               name: str, 
               version: str, 
               release_type: str, 
               release_date: str,
               files: List[Any], # List of open file objects or tuples
               changelog: str = "",
               tags: Optional[List[str]] = None,
               dependencies: Optional[List[Dict[str, Any]]] = None) -> ProjectVersion:
        """Create a new project version
        
        Args:
            dependencies: List of dependency dicts with keys: project, version, type, external
        """
        data = {
            'name': name,
            'version': version,
            'release_type': release_type,
            'release_date': release_date,
            'changelog': changelog,
        }
        
        # Build form data as tuples
        data_tuples = []
        for k, v in data.items():
            if v:  # Only add non-empty values
                data_tuples.append((k, v))
        
        if tags:
            for tag in tags:
                data_tuples.append(('tags[]', tag))
        
        # Dependencies need to be sent as indexed fields: dependencies[0][project], etc.
        if dependencies:
            for i, dep in enumerate(dependencies):
                for key, value in dep.items():
                    # Convert boolean to string for form data
                    if isinstance(value, bool):
                        value = '1' if value else '0'
                    data_tuples.append((f'dependencies[{i}][{key}]', str(value)))

        # Files need to be a list of ('files[]', file_obj)
        files_payload = []
        for f in files:
            files_payload.append(('files[]', f))

        response = self._request('POST', f'/v1/project/{slug}/versions', data=data_tuples, files=files_payload)
        return ProjectVersion.from_dict(response.get('data'))
    
    def update(self, slug: str, version: str,
               name: Optional[str] = None,
               version_new: Optional[str] = None,
               release_type: Optional[str] = None,
               release_date: Optional[str] = None,
               changelog: Optional[str] = None,
               tags: Optional[List[str]] = None,
               files: Optional[List[Any]] = None,
               files_to_remove: Optional[List[str]] = None,
               clean_existing_files: bool = False,
               dependencies: Optional[List[Dict[str, Any]]] = None) -> ProjectVersion:
        """Update an existing project version
        
        Args:
            version: Current version slug to update
            version_new: New version number if changing the version
            dependencies: List of dependency dicts with keys: project, version, type, external
            
        Note: The API requires name, version, release_type, and release_date 
              fields even when updating. Pass at least these fields.
        """
        data = {}
        
        # Always include version (use new version if provided, otherwise use current)
        data['version'] = version_new if version_new else version
        
        if name:
            data['name'] = name
        if release_type:
            data['release_type'] = release_type
        if release_date:
            data['release_date'] = release_date
        if changelog is not None:
            data['changelog'] = changelog
        if clean_existing_files:
            data['clean_existing_files'] = '1'
            
        data_tuples = []
        for k, v in data.items():
            data_tuples.append((k, v))
            
        if tags:
            for tag in tags:
                data_tuples.append(('tags[]', tag))
        
        if dependencies:
            for i, dep in enumerate(dependencies):
                for key, value in dep.items():
                    if isinstance(value, bool):
                        value = '1' if value else '0'
                    data_tuples.append((f'dependencies[{i}][{key}]', str(value)))
                
        if files_to_remove:
            for f in files_to_remove:
                data_tuples.append(('files_to_remove[]', f))
        
        files_payload = []
        if files:
            for f in files:
                files_payload.append(('files[]', f))
        
        response = self._request('POST', f'/v1/project/{slug}/version/{version}', data=data_tuples, files=files_payload if files_payload else None)
        return ProjectVersion.from_dict(response.get('data'))
    
    def delete(self, slug: str, version: str) -> None:
        """Delete a project version"""
        self._request('DELETE', f'/v1/project/{slug}/version/{version}')
        return None

class TagsClient(BaseClient):
    def list_project_tags(self, plain: bool = False, project_type: Optional[str] = None) -> List[ProjectTag]:
        params = {'plain': plain}
        if project_type:
            params['project_type'] = project_type
        response = self._request('GET', '/v1/project_tags', params=params)
        return [ProjectTag.from_dict(item) for item in response.get('data', [])]

    def get_project_tag(self, slug: str) -> ProjectTag:
        response = self._request('GET', f'/v1/project_tag/{slug}')
        return ProjectTag.from_dict(response.get('data'))

    def list_version_tags(self, plain: bool = False, project_type: Optional[str] = None) -> List[ProjectTag]: # Reusing ProjectTag as structure seems similar enough or should create ProjectVersionTag
        params = {'plain': plain}
        if project_type:
            params['project_type'] = project_type
        response = self._request('GET', '/v1/version_tags', params=params)
        # Spec says ProjectVersionTagResource, which is identical structure to ProjectTagResource in properties shown
        return [ProjectTag.from_dict(item) for item in response.get('data', [])]

    def get_version_tag(self, slug: str) -> ProjectTag:
        response = self._request('GET', f'/v1/version_tag/{slug}')
        return ProjectTag.from_dict(response.get('data'))

class UsersClient(BaseClient):
    def get(self, name: str) -> User:
        response = self._request('GET', f'/v1/user/{name}')
        return User.from_dict(response.get('data'))

    def get_projects(self, name: str) -> Dict[str, Any]:
        response = self._request('GET', f'/v1/user/{name}/projects')
        projects = [Project.from_dict(item) for item in response.get('data', [])]
        response['data'] = projects
        return response

class HubClient(BaseClient):
    def __init__(self, base_url: str, token: Optional[str] = None):
        super().__init__(base_url, token)
        self.project_types = ProjectTypesClient(base_url, token)
        self.projects = ProjectsClient(base_url, token)
        self.versions = ProjectVersionsClient(base_url, token)
        self.tags = TagsClient(base_url, token)
        self.users = UsersClient(base_url, token)

    def test_token(self) -> Dict[str, Any]:
        return self._request('GET', '/test-token')
