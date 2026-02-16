from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any

@dataclass
class ProjectType:
    name: str
    slug: str
    icon: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ProjectType':
        return cls(
            name=data['name'],
            slug=data['slug'],
            icon=data['icon']
        )

@dataclass
class ProjectTag:
    name: str
    slug: str
    icon: str
    tag_group: Optional[str]
    project_types: List[str]
    main_tag: Optional[str]
    sub_tags: List['ProjectTag'] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ProjectTag':
        sub_tags = [cls.from_dict(t) for t in data.get('sub_tags', [])] if 'sub_tags' in data else []
        return cls(
            name=data['name'],
            slug=data['slug'],
            icon=data['icon'],
            tag_group=data.get('tag_group'),
            project_types=data['project_types'],
            main_tag=data.get('main_tag'),
            sub_tags=sub_tags
        )

@dataclass
class ProjectVersionTag:
    name: str
    slug: str
    icon: str
    tag_group: Optional[str]
    project_types: List[str]
    main_tag: Optional[str]
    sub_tags: List['ProjectVersionTag'] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ProjectVersionTag':
        sub_tags = [cls.from_dict(t) for t in data.get('sub_tags', [])] if 'sub_tags' in data else []
        return cls(
            name=data['name'],
            slug=data['slug'],
            icon=data['icon'],
            tag_group=data.get('tag_group'),
            project_types=data['project_types'],
            main_tag=data.get('main_tag'),
            sub_tags=sub_tags
        )

@dataclass
class Project:
    name: str
    slug: str
    summary: str
    description: Optional[str]
    logo_url: str
    website: Optional[str]
    issues: Optional[str]
    source: Optional[str]
    status: str
    downloads: int
    created_at: str # Keep as string for simplicity, or parse to datetime
    last_release_date: Optional[str]
    updated_at: Optional[str] = None  # Last update time of the project or one of its versions
    version_count: int = 0
    tags: List[str] = field(default_factory=list)
    members: List[Dict[str, Any]] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Project':
        return cls(
            name=data['name'],
            slug=data['slug'],
            summary=data['summary'],
            description=data.get('description'),
            logo_url=data['logo_url'],
            website=data.get('website'),
            issues=data.get('issues'),
            source=data.get('source'),
            status=data['status'],
            downloads=data['downloads'],
            created_at=data['created_at'],
            last_release_date=data.get('last_release_date'),
            updated_at=data.get('updated_at'),
            version_count=data.get('version_count', 0),
            tags=data.get('tags') or [], # Handle null
            members=data.get('members', [])
        )

@dataclass
class ProjectFile:
    name: str
    size: int
    sha1: str
    url: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ProjectFile':
        return cls(
            name=data['name'],
            size=data['size'],
            sha1=data['sha1'],
            url=data['url']
        )

@dataclass
class ProjectVersionDependency:
    project_slug: str
    version_slug: Optional[str]
    type: str # required, optional, embedded
    external: bool

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ProjectVersionDependency':
        return cls(
            project_slug=data['project'],
            version_slug=data.get('version'),
            type=data['type'],
            external=data['external']
        )

@dataclass
class ProjectVersion:
    name: str
    version: str
    release_type: str
    release_date: str
    changelog: Optional[str]
    downloads: int
    tags: List[str]
    files: List[ProjectFile] = field(default_factory=list)
    dependencies: List[ProjectVersionDependency] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ProjectVersion':
        files = [ProjectFile.from_dict(f) for f in data.get('files', [])]
        dependencies = [ProjectVersionDependency.from_dict(d) for d in data.get('dependencies', [])]
        return cls(
            name=data['name'],
            version=data['version'],
            release_type=data['release_type'],
            release_date=data['release_date'],
            changelog=data.get('changelog'),
            downloads=data['downloads'],
            tags=data.get('tags', []),
            files=files,
            dependencies=dependencies
        )

@dataclass
class User:
    username: str
    bio: Optional[str]
    avatar: Optional[str]
    created_at: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'User':
        return cls(
            username=data['username'],
            bio=data.get('bio'),
            avatar=data.get('avatar'),
            created_at=data['created_at']
        )
