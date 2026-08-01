import os
import shutil
import subprocess
import configparser
from typing import Optional
from domain.theme.entities import (
    ThemeState,
    AppearanceSettings,
    AvailableAppearanceOptions,
)
from domain.theme.repositories import IThemeRepository
from domain.notification.entities import Notification
from domain.notification.repositories import INotificationRepository
from infrastructure.notification.desktop_notification_repository import (
    DesktopNotificationRepository,
)


class GtkQtThemeRepository(IThemeRepository):
    WORK_PATH = os.path.expanduser("~/.config/theme_toggle")
    FILE = os.path.join(WORK_PATH, "theme")
    FOOT_DIR = os.path.expanduser("~/.config/foot")
    FOOT_FILE = os.path.join(FOOT_DIR, "foot.ini")

    def __init__(self, notification_repo: Optional[INotificationRepository] = None):
        self.notification_repo = notification_repo or DesktopNotificationRepository()


    def get_state(self) -> ThemeState:
        if os.path.isfile(self.FILE):
            try:
                with open(self.FILE, "r") as f:
                    return ThemeState(current_theme=f.read().strip())
            except Exception:
                pass
        return ThemeState(current_theme="light")

    def toggle(self) -> str:
        current = self.get_state().current_theme
        new_theme = "dark" if current == "light" else "light"

        gtk4 = "prefer-dark" if new_theme == "dark" else "prefer-light"
        gtk3 = "Adwaita-dark" if new_theme == "dark" else "Adwaita"
        foot_theme = "black.ini" if new_theme == "dark" else "white.ini"

        # Apply GTK theme
        subprocess.run(
            [
                "gsettings",
                "set",
                "org.gnome.desktop.interface",
                "color-scheme",
                gtk4,
            ]
        )
        subprocess.run(
            ["gsettings", "set", "org.gnome.desktop.interface", "gtk-theme", gtk3]
        )

        # Apply Foot terminal theme
        theme_src = os.path.join(self.FOOT_DIR, "themes", foot_theme)
        if os.path.isfile(theme_src):
            try:
                if os.path.isfile(self.FOOT_FILE) or os.path.islink(
                    self.FOOT_FILE
                ):
                    os.remove(self.FOOT_FILE)
                shutil.copyfile(theme_src, self.FOOT_FILE)
            except Exception as e:
                print(f"Error copying foot theme: {e}")

        # Apply Qt themes
        self._apply_qt_theme(new_theme)

        os.makedirs(self.WORK_PATH, exist_ok=True)
        with open(self.FILE, "w") as f:
            f.write(new_theme)

        msg = f"Trocando para o tema {new_theme}"
        self.notification_repo.notify(Notification(title=msg))
        return msg

    def get_appearance_settings(self) -> AppearanceSettings:
        gtk_theme = self._get_gsetting("gtk-theme") or self._get_ini_setting("gtk-3.0", "gtk-theme-name") or "Adwaita"
        icon_theme = self._get_gsetting("icon-theme") or self._get_ini_setting("gtk-3.0", "gtk-icon-theme-name") or "Adwaita"
        cursor_theme = self._get_gsetting("cursor-theme") or self._get_ini_setting("gtk-3.0", "gtk-cursor-theme-name") or "Adwaita"
        font_name = self._get_gsetting("font-name") or self._get_ini_setting("gtk-3.0", "gtk-font-name") or "Sans 10"

        return AppearanceSettings(
            gtk_theme=gtk_theme,
            icon_theme=icon_theme,
            cursor_theme=cursor_theme,
            font_name=font_name,
        )

    def get_available_options(self) -> AvailableAppearanceOptions:
        return AvailableAppearanceOptions(
            gtk_themes=self._scan_gtk_themes(),
            icon_themes=self._scan_icon_themes(),
            cursor_themes=self._scan_cursor_themes(),
            fonts=self._scan_system_fonts(),
        )

    def apply_appearance(self, settings: AppearanceSettings) -> bool:
        try:
            # 1. Update gsettings
            if settings.gtk_theme:
                subprocess.run(["gsettings", "set", "org.gnome.desktop.interface", "gtk-theme", settings.gtk_theme])
                # Auto adjust color-scheme if GTK4 / Gnome theme indicates dark
                color_scheme = "prefer-dark" if "dark" in settings.gtk_theme.lower() else "default"
                subprocess.run(["gsettings", "set", "org.gnome.desktop.interface", "color-scheme", color_scheme])
            if settings.icon_theme:
                subprocess.run(["gsettings", "set", "org.gnome.desktop.interface", "icon-theme", settings.icon_theme])
            if settings.cursor_theme:
                subprocess.run(["gsettings", "set", "org.gnome.desktop.interface", "cursor-theme", settings.cursor_theme])
            if settings.font_name:
                subprocess.run(["gsettings", "set", "org.gnome.desktop.interface", "font-name", settings.font_name])

            # 2. Update GTK 3.0 & GTK 4.0 settings.ini
            for ver in ["gtk-3.0", "gtk-4.0"]:
                path = os.path.expanduser(f"~/.config/{ver}/settings.ini")
                if settings.gtk_theme:
                    self._update_ini_setting(path, "Settings", "gtk-theme-name", settings.gtk_theme)
                if settings.icon_theme:
                    self._update_ini_setting(path, "Settings", "gtk-icon-theme-name", settings.icon_theme)
                if settings.cursor_theme:
                    self._update_ini_setting(path, "Settings", "gtk-cursor-theme-name", settings.cursor_theme)
                if settings.font_name:
                    self._update_ini_setting(path, "Settings", "gtk-font-name", settings.font_name)

            # 3. Update ~/.gtkrc-2.0
            self._update_gtk2_rc(settings)

            # 4. Update ~/.icons/default/index.theme for cursor fallback
            if settings.cursor_theme:
                self._update_cursor_index_theme(settings.cursor_theme)

            self.notification_repo.notify(
                Notification(
                    title="SwayManager",
                    message=f"Aparência aplicada!\nTema: {settings.gtk_theme}",
                )
            )
            return True
        except Exception as e:
            print(f"Error applying appearance settings: {e}")
            return False

    def _get_gsetting(self, key: str) -> str:
        try:
            res = subprocess.run(
                ["gsettings", "get", "org.gnome.desktop.interface", key],
                capture_output=True,
                text=True,
                check=True,
            )
            val = res.stdout.strip().strip("'\"")
            return val
        except Exception:
            return ""

    def _get_ini_setting(self, config_dir: str, key: str) -> str:
        try:
            path = os.path.expanduser(f"~/.config/{config_dir}/settings.ini")
            if os.path.isfile(path):
                config = configparser.ConfigParser(interpolation=None)
                config.read(path)
                if "Settings" in config and key in config["Settings"]:
                    return config["Settings"][key]
        except Exception:
            pass
        return ""

    def _scan_gtk_themes(self) -> list[str]:
        search_paths = [
            "/usr/share/themes",
            "/usr/local/share/themes",
            os.path.expanduser("~/.themes"),
            os.path.expanduser("~/.local/share/themes"),
        ]
        themes = set()
        for base in search_paths:
            if not os.path.isdir(base):
                continue
            for item in os.listdir(base):
                full_path = os.path.join(base, item)
                if not os.path.isdir(full_path):
                    continue
                # Check if it has GTK markers or index.theme
                has_gtk = any(
                    os.path.exists(os.path.join(full_path, sub))
                    for sub in ["gtk-2.0", "gtk-3.0", "gtk-4.0", "index.theme"]
                )
                if has_gtk:
                    themes.add(item)

        return sorted(list(themes))

    def _scan_icon_themes(self) -> list[str]:
        search_paths = [
            "/usr/share/icons",
            "/usr/local/share/icons",
            os.path.expanduser("~/.icons"),
            os.path.expanduser("~/.local/share/icons"),
        ]
        icons = set()
        ignore = {"default", "vendor", "distrobox", "zbar.ico"}

        for base in search_paths:
            if not os.path.isdir(base):
                continue
            for item in os.listdir(base):
                if item in ignore or item.startswith("."):
                    continue
                full_path = os.path.join(base, item)
                if not os.path.isdir(full_path):
                    continue
                index_theme = os.path.join(full_path, "index.theme")
                if os.path.isfile(index_theme):
                    # Check if it's an icon theme (not cursor-only)
                    try:
                        with open(index_theme, "r", encoding="utf-8", errors="ignore") as f:
                            content = f.read()
                            if "[Icon Theme]" in content:
                                # If it has subdirectories or non-cursor icon directories
                                icons.add(item)
                    except Exception:
                        pass

        return sorted(list(icons))

    def _scan_cursor_themes(self) -> list[str]:
        search_paths = [
            "/usr/share/icons",
            "/usr/local/share/icons",
            os.path.expanduser("~/.icons"),
            os.path.expanduser("~/.local/share/icons"),
        ]
        cursors = set()

        for base in search_paths:
            if not os.path.isdir(base):
                continue
            for item in os.listdir(base):
                if item.startswith("."):
                    continue
                full_path = os.path.join(base, item)
                if not os.path.isdir(full_path):
                    continue
                # Check for cursors directory
                has_cursors = os.path.isdir(os.path.join(full_path, "cursors"))
                if has_cursors:
                    cursors.add(item)

        return sorted(list(cursors))

    def _scan_system_fonts(self) -> list[str]:
        fonts = set()
        try:
            res = subprocess.run(
                ["fc-list", ":", "family"],
                capture_output=True,
                text=True,
            )
            if res.returncode == 0 and res.stdout:
                for line in res.stdout.splitlines():
                    for name in line.split(","):
                        clean_name = name.strip()
                        if clean_name and not clean_name.startswith("."):
                            fonts.add(clean_name)
        except Exception as e:
            print(f"Error listing fonts via fc-list: {e}")

        if not fonts:
            fonts = {"Sans", "Serif", "Monospace", "DejaVu Sans", "Ubuntu", "Inter", "Roboto"}

        return sorted(list(fonts))

    def _update_gtk2_rc(self, settings: AppearanceSettings):
        path = os.path.expanduser("~/.gtkrc-2.0")
        lines = []
        if os.path.isfile(path):
            try:
                with open(path, "r", encoding="utf-8", errors="ignore") as f:
                    lines = f.readlines()
            except Exception:
                pass

        kv_map = {
            "gtk-theme-name": settings.gtk_theme,
            "gtk-icon-theme-name": settings.icon_theme,
            "gtk-cursor-theme-name": settings.cursor_theme,
            "gtk-font-name": settings.font_name,
        }

        new_lines = []
        found_keys = set()

        for line in lines:
            stripped = line.strip()
            updated = False
            for k, v in kv_map.items():
                if stripped.startswith(f'{k}='):
                    new_lines.append(f'{k}="{v}"\n')
                    found_keys.add(k)
                    updated = True
                    break
            if not updated:
                new_lines.append(line)

        for k, v in kv_map.items():
            if k not in found_keys and v:
                new_lines.append(f'{k}="{v}"\n')

        try:
            with open(path, "w", encoding="utf-8") as f:
                f.writelines(new_lines)
        except Exception as e:
            print(f"Error writing ~/.gtkrc-2.0: {e}")

    def _update_cursor_index_theme(self, cursor_theme: str):
        path = os.path.expanduser("~/.icons/default/index.theme")
        os.makedirs(os.path.dirname(path), exist_ok=True)
        content = f"[Icon Theme]\nInherits={cursor_theme}\n"
        try:
            with open(path, "w", encoding="utf-8") as f:
                f.write(content)
        except Exception as e:
            print(f"Error writing default cursor index.theme: {e}")

    def _update_ini_setting(self, filepath: str, section: str, key: str, value: str):
        try:
            os.makedirs(os.path.dirname(filepath), exist_ok=True)
            config = configparser.ConfigParser(interpolation=None)
            config.optionxform = str
            if os.path.isfile(filepath):
                config.read(filepath)
            if section not in config:
                config[section] = {}
            config[section][key] = value
            with open(filepath, "w") as f:
                config.write(f)
        except Exception as e:
            print(f"Error updating INI file {filepath}: {e}")

    def _apply_qt_theme(self, new_theme: str):
        qt_style = "adwaita-dark" if new_theme == "dark" else "adwaita"
        kv_theme = "KvAdwaitaDark" if new_theme == "dark" else "KvAdwaita"
        kde_scheme = "BreezeDark" if new_theme == "dark" else "BreezeLight"

        for conf_dir in ["~/.config/qt5ct", "~/.config/qt6ct"]:
            tool_name = os.path.basename(conf_dir)
            conf_dir_expanded = os.path.expanduser(conf_dir)
            conf_path = os.path.join(conf_dir_expanded, f"{tool_name}.conf")
            if os.path.isdir(conf_dir_expanded) or os.path.isfile(conf_path) or shutil.which(tool_name):
                self._update_ini_setting(conf_path, "Appearance", "style", qt_style)

        kv_dir = os.path.expanduser("~/.config/Kvantum")
        kv_path = os.path.join(kv_dir, "kvantum.kvconfig")
        if os.path.isdir(kv_dir) or os.path.isfile(kv_path) or shutil.which("kvantummanager"):
            self._update_ini_setting(kv_path, "General", "theme", kv_theme)
            if shutil.which("kvantummanager"):
                try:
                    subprocess.run(
                        ["kvantummanager", "--set", kv_theme],
                        stderr=subprocess.DEVNULL,
                        stdout=subprocess.DEVNULL,
                    )
                except Exception as e:
                    print(f"Error calling kvantummanager: {e}")

        kde_path = os.path.expanduser("~/.config/kdeglobals")
        if os.path.isfile(kde_path) or shutil.which("plasma-apply-colorscheme"):
            self._update_ini_setting(kde_path, "General", "ColorScheme", kde_scheme)
            if shutil.which("plasma-apply-colorscheme"):
                try:
                    subprocess.run(
                        ["plasma-apply-colorscheme", kde_scheme],
                        stderr=subprocess.DEVNULL,
                        stdout=subprocess.DEVNULL,
                    )
                except Exception as e:
                    print(f"Error calling plasma-apply-colorscheme: {e}")

