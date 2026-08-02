from dataclasses import dataclass
from typing import Optional


@dataclass
class MenuItem:
    name: str
    normalized_name: str
    exec_cmd: str
    icon: str
    category: str
    comment: Optional[str] = ""
    is_system_action: bool = False
    is_category_header: bool = False
    category_target: Optional[str] = None
    is_web_app: bool = False

