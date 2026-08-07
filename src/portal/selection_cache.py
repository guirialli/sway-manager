"""Persistent selection cache for the portal chooser.

When xdg-desktop-portal-wlr invokes the chooser multiple times within a short
window (some clients like Discord/Chromium retry SelectSources / Start), the
same set of sources is presented again.  This module remembers the user's last
pick and, if the sources haven't changed, replays it silently without showing
the GUI again.
"""

import json
import os
import time
from dataclasses import dataclass, field

from portal.models import PortalResult, PortalSource

CACHE_DIR = os.path.expanduser("~/.cache/sway-manager")
CACHE_FILE = os.path.join(CACHE_DIR, "portal_selection.json")
CACHE_TTL_SECONDS = 30


@dataclass
class _CacheEntry:
    raw_label: str
    source_type: str
    source_id: str
    source_hash: str
    timestamp: float = field(default_factory=time.time)


def _sources_hash(monitors: list[PortalSource], windows: list[PortalSource]) -> str:
    """Fast stable hash — native Python hash, not crypto."""
    ids = tuple(
        sorted(
            (s.source_type.value, s.raw_label or s.id)
            for s in (monitors + windows)
        )
    )
    return str(hash(ids))

def _load_cache() -> _CacheEntry | None:
    try:
        with open(CACHE_FILE, "r") as fh:
            data = json.load(fh)
        entry = _CacheEntry(
            raw_label=data["raw_label"],
            source_type=data["source_type"],
            source_id=data["source_id"],
            source_hash=data["source_hash"],
            timestamp=data["timestamp"],
        )
        if time.time() - entry.timestamp > CACHE_TTL_SECONDS:
            return None
        return entry
    except (FileNotFoundError, json.JSONDecodeError, KeyError):
        return None


def _save_cache(result: PortalResult, source_hash: str) -> None:
    os.makedirs(CACHE_DIR, exist_ok=True)
    data = {
        "raw_label": str(result),
        "source_type": result.source_type.value,
        "source_id": result.id,
        "source_hash": source_hash,
        "timestamp": time.time(),
    }
    with open(CACHE_FILE, "w") as fh:
        json.dump(data, fh)


def try_replay(
    monitors: list[PortalSource], windows: list[PortalSource]
) -> PortalResult | None:
    entry = _load_cache()
    if entry is None:
        return None

    current_hash = _sources_hash(monitors, windows)
    if current_hash != entry.source_hash:
        return None

    from portal.models import PortalSourceType

    st = (
        PortalSourceType.MONITOR
        if entry.source_type == "monitor"
        else PortalSourceType.WINDOW
    )
    return PortalResult(source_type=st, id=entry.source_id, raw_label=entry.raw_label)


def store_selection(
    result: PortalResult,
    monitors: list[PortalSource],
    windows: list[PortalSource],
) -> None:
    _save_cache(result, _sources_hash(monitors, windows))


def clear_cache() -> None:
    try:
        os.remove(CACHE_FILE)
    except FileNotFoundError:
        pass
