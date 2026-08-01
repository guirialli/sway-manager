from dataclasses import dataclass
from enum import Enum


class DisplaySwitchType(Enum):
    PC_ONLY = "0"
    DUPLICATE = "1"
    EXTEND = "2"
    MONITOR_ONLY = "3"


@dataclass(frozen=True)
class MonitoresSway:
    interno: str
    externo: str
