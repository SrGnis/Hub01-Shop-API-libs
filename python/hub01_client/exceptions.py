class HubAPIException(Exception):
    """Base exception for Hub01 Shop API"""
    pass

class AuthenticationException(HubAPIException):
    """Raised when authentication fails (401)"""
    pass

class PermissionDeniedException(HubAPIException):
    """Raised when permission is denied (403)"""
    pass

class NotFoundException(HubAPIException):
    """Raised when a resource is not found (404)"""
    pass

class ValidationException(HubAPIException):
    """Raised when validation fails (422)"""
    def __init__(self, message, errors=None):
        super().__init__(message)
        self.errors = errors
