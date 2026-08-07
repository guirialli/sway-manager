"""Portal integration package for XDG Desktop Portal screen sharing."""

from portal.models import (
    PortalResult,
    PortalSource,
    PortalSourceType,
    WindowSharingAvailability,
)
from portal.result_writer import PortalResultWriter
from portal.outputs_provider import SwayOutputsProvider
from portal.windows_provider import WindowSharingProvider
from portal.diagnostics import PortalDiagnostics

from portal.exceptions import PortalException, SwayNotAvailableError

__all__ = [
    "PortalResult",
    "PortalSource",
    "PortalSourceType",
    "WindowSharingAvailability",
    "PortalResultWriter",
    "SwayOutputsProvider",
    "WindowSharingProvider",
    "PortalDiagnostics",
    "PortalController",
    "PortalException",
    "SwayNotAvailableError",
]


def __getattr__(name: str):
    if name == "PortalController":
        from portal.controller import PortalController

        return PortalController
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")
