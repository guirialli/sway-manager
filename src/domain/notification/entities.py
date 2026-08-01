from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class Notification:
    title: str
    message: str = ""
    urgency: str = "normal"  # "low", "normal", "critical"
    icon: Optional[str] = None
