class PortalException(Exception):
    """Base exception for portal integration errors."""

    def __init__(self, message: str) -> None:
        super().__init__(message)
        self.message = message


class SwayNotAvailableError(PortalException):
    """Raised when swaymsg cannot be executed or returns no usable outputs."""

    pass


class WindowDiscoveryError(PortalException):
    """Raised when window discovery fails in an environment that should support it."""

    pass
