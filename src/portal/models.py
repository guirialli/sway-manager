from dataclasses import dataclass, field
from enum import Enum


class PortalSourceType(Enum):
    MONITOR = "monitor"
    WINDOW = "window"


@dataclass(frozen=True)
class PortalSource:
    """A shareable source discovered by the portal selector.

    Attributes:
        id: Technical identifier returned to xdg-desktop-portal-wlr.
        source_type: Whether this source is a monitor or a window.
        label: Human-friendly name shown in the UI.
        details: Multi-line description shown in the UI.
        is_primary: Whether the source is the primary display.
        is_focused: Whether the source currently has focus.
        extra: Additional provider-specific metadata (not logged by default).
    """

    id: str
    source_type: PortalSourceType
    label: str
    details: str
    is_primary: bool = False
    is_focused: bool = False
    extra: dict = field(default_factory=dict)
    raw_label: str | None = None

@dataclass(frozen=True)
class WindowSharingAvailability:
    """Whether individual window capture is usable in this session."""

    supported: bool
    reason: str | None = None


@dataclass(frozen=True)
class PortalResult:
    """Final selection emitted to the portal backend."""

    source_type: PortalSourceType
    id: str
    raw_label: str | None = None

    def __str__(self) -> str:
        if self.raw_label is not None:
            return self.raw_label
        return self.id
