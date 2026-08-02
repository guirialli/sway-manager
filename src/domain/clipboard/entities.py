from dataclasses import dataclass
from typing import Optional


@dataclass
class ClipboardItem:
    id: str
    text: str
    raw_preview: str
    is_image: bool = False
    image_path: Optional[str] = None
    is_favorite: bool = False
    is_action: bool = False
    action_type: Optional[str] = None  # "clear", "manage_favorites", "pin", "unpin"
