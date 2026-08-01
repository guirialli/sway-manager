from dataclasses import dataclass, field
from typing import Dict, Any, List


@dataclass(frozen=True)
class AppearanceSettings:
    gtk_theme: str = "Adwaita"
    icon_theme: str = "Adwaita"
    cursor_theme: str = "Adwaita"
    font_name: str = "Sans 10"


@dataclass(frozen=True)
class AvailableAppearanceOptions:
    gtk_themes: List[str] = field(default_factory=list)
    icon_themes: List[str] = field(default_factory=list)
    cursor_themes: List[str] = field(default_factory=list)
    fonts: List[str] = field(default_factory=list)



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
    # Greeter - Aparência
    background: str = "/usr/share/backgrounds/lightdm-wallpaper.jpg"
    theme_name: str = "Adwaita-dark"
    icon_theme_name: str = "Adwaita"
    font_name: str = "Sans 11"
    cursor_theme_name: str = "Bibata-Modern-Ice"
    cursor_theme_size: str = "24"
    xft_antialias: str = "true"
    xft_dpi: str = ""
    xft_hintstyle: str = "hintslight"
    xft_rgba: str = "rgb"

    # Greeter - Layout, Relógio e Indicadores
    clock_format: str = "%a, %d %b %H:%M"
    position: str = "50%,center"
    active_monitor: str = "#cursor"
    screensaver_timeout: str = "60"
    indicators: str = "~host;~spacer;~clock;~spacer;~session;~language;~a11y;~power"

    # Greeter - Exibição e Usuários
    draw_user_backgrounds: str = "false"
    draw_grid: str = "false"
    hide_user_image: str = "false"
    default_user_image: str = ""

    # Daemon / Seat - Autologin & Sessão
    autologin_user: str = ""
    autologin_user_timeout: str = "0"
    autologin_session: str = "sway"
    user_session: str = "sway"
    greeter_show_manual_login: str = "false"
    greeter_hide_users: str = "false"
    allow_guest: str = "false"

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "LightDMSettings":
        return cls(
            background=data.get("background", cls.background),
            theme_name=data.get("theme-name", cls.theme_name),
            icon_theme_name=data.get("icon-theme-name", cls.icon_theme_name),
            font_name=data.get("font-name", cls.font_name),
            cursor_theme_name=data.get("cursor-theme-name", cls.cursor_theme_name),
            cursor_theme_size=str(data.get("cursor-theme-size", cls.cursor_theme_size)),
            xft_antialias=str(data.get("xft-antialias", cls.xft_antialias)),
            xft_dpi=str(data.get("xft-dpi", cls.xft_dpi)),
            xft_hintstyle=data.get("xft-hintstyle", cls.xft_hintstyle),
            xft_rgba=data.get("xft-rgba", cls.xft_rgba),
            clock_format=data.get("clock-format", cls.clock_format),
            position=data.get("position", cls.position),
            active_monitor=data.get("active-monitor", cls.active_monitor),
            screensaver_timeout=str(data.get("screensaver-timeout", cls.screensaver_timeout)),
            indicators=data.get("indicators", cls.indicators),
            draw_user_backgrounds=str(data.get("draw-user-backgrounds", cls.draw_user_backgrounds)),
            draw_grid=str(data.get("draw-grid", cls.draw_grid)),
            hide_user_image=str(data.get("hide-user-image", cls.hide_user_image)),
            default_user_image=data.get("default-user-image", cls.default_user_image),
            autologin_user=data.get("autologin-user", cls.autologin_user),
            autologin_user_timeout=str(data.get("autologin-user-timeout", cls.autologin_user_timeout)),
            autologin_session=data.get("autologin-session", cls.autologin_session),
            user_session=data.get("user-session", cls.user_session),
            greeter_show_manual_login=str(data.get("greeter-show-manual-login", cls.greeter_show_manual_login)),
            greeter_hide_users=str(data.get("greeter-hide-users", cls.greeter_hide_users)),
            allow_guest=str(data.get("allow-guest", cls.allow_guest)),
        )

    def to_dict(self) -> Dict[str, str]:
        return {
            "background": self.background,
            "theme-name": self.theme_name,
            "icon-theme-name": self.icon_theme_name,
            "font-name": self.font_name,
            "cursor-theme-name": self.cursor_theme_name,
            "cursor-theme-size": self.cursor_theme_size,
            "xft-antialias": self.xft_antialias,
            "xft-dpi": self.xft_dpi,
            "xft-hintstyle": self.xft_hintstyle,
            "xft-rgba": self.xft_rgba,
            "clock-format": self.clock_format,
            "position": self.position,
            "active-monitor": self.active_monitor,
            "screensaver-timeout": self.screensaver_timeout,
            "indicators": self.indicators,
            "draw-user-backgrounds": self.draw_user_backgrounds,
            "draw-grid": self.draw_grid,
            "hide-user-image": self.hide_user_image,
            "default-user-image": self.default_user_image,
            "autologin-user": self.autologin_user,
            "autologin-user-timeout": self.autologin_user_timeout,
            "autologin-session": self.autologin_session,
            "user-session": self.user_session,
            "greeter-show-manual-login": self.greeter_show_manual_login,
            "greeter-hide-users": self.greeter_hide_users,
            "allow-guest": self.allow_guest,
        }

