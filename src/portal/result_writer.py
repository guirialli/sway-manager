import sys
from typing import TextIO

from portal.models import PortalResult, PortalSourceType


class PortalResultWriter:
    """Writes portal selection results to stdout.

    This is the only component allowed to write the result to stdout.  All
    diagnostic or logging output must be directed to stderr or to a log file so
    the contract with xdg-desktop-portal-wlr is not broken.
    """

    def __init__(self, out: TextIO = sys.stdout) -> None:
        self._out = out

    def write_result(self, result: PortalResult) -> None:
        """Write the selected source in the format expected by the portal."""
        line = str(result)
        if not line:
            return
        self._out.write(line + "\n")
        self._out.flush()

    def write_monitor(self, output_name: str) -> None:
        """Convenience helper for monitor results."""
        self.write_result(PortalResult(PortalSourceType.MONITOR, output_name))

    def write_window(self, window_id: str) -> None:
        """Convenience helper for window results."""
        self.write_result(PortalResult(PortalSourceType.WINDOW, window_id))

    def cancel(self) -> None:
        """User cancelled: emit nothing to stdout."""
        pass

    @staticmethod
    def validate_identifier(identifier: str) -> bool:
        """Return True if the identifier is safe to emit."""
        if not identifier:
            return False
        # Reject values that would break the simple line-based contract or
        # expose accidental multi-line identifiers.
        return "\n" not in identifier and "\r" not in identifier
