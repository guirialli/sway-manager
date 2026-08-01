from dataclasses import dataclass
from enum import Enum


class AudioAction(Enum):
    UP = "up"
    DOWN = "down"
    MUTE = "mute"


@dataclass(frozen=True)
class VolumeState:
    percentage: float
    is_muted: bool
