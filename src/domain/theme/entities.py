from dataclasses import dataclass
from typing import Dict, Any


@dataclass(frozen=True)
class ThemeState:
    current_theme: str  # "light" or "dark"

    def to_waybar_json(self) -> dict:
        if self.current_theme == "light":
            return {
                "text": "☀️",
                "tooltip": "Tema Light",
                "class": "theme-light",
            }
        return {
            "text": "🌕",
            "tooltip": "Tema Dark",
            "class": "theme-dark",
        }


@dataclass(frozen=True)
class LightDMSettings:
    background: str = "/usr/share/backgrounds/lightdm-wallpaper.jpg"
    theme_name: str = "Adwaita-dark"
    icon_theme_name: str = "Adwaita"
    font_name: str = "Sans 11"
    cursor_theme_name: str = "Bibata-Modern-Ice"
    clock_format: str = "%a, %d %b %H:%M"
    draw_user_backgrounds: str = "false"
    hide_user_image: str = "false"

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "LightDMSettings":
        return cls(
            background=data.get("background", cls.background),
            theme_name=data.get("theme-name", cls.theme_name),
            icon_theme_name=data.get("icon-theme-name", cls.icon_theme_name),
            font_name=data.get("font-name", cls.font_name),
            cursor_theme_name=data.get("cursor-theme-name", cls.cursor_theme_name),
            clock_format=data.get("clock-format", cls.clock_format),
            draw_user_backgrounds=data.get("draw-user-backgrounds", cls.draw_user_backgrounds),
            hide_user_image=data.get("hide-user-image", cls.hide_user_image),
        )

    def to_dict(self) -> Dict[str, str]:
        return {
            "background": self.background,
            "theme-name": self.theme_name,
            "icon-theme-name": self.icon_theme_name,
            "font-name": self.font_name,
            "cursor-theme-name": self.cursor_theme_name,
            "clock-format": self.clock_format,
            "draw-user-backgrounds": self.draw_user_backgrounds,
            "hide-user-image": self.hide_user_image,
        }
