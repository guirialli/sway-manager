import os
import subprocess
import configparser
from typing import Optional


class IconResolver:
    _gtk_theme = None
    _gtk_initialized = False

    @classmethod
    def get_system_icon_theme_name(cls) -> str:
        # 1. Try gsettings
        try:
            res = subprocess.run(
                ["gsettings", "get", "org.gnome.desktop.interface", "icon-theme"],
                capture_output=True,
                text=True,
                timeout=2,
            )
            val = res.stdout.strip().strip("'\"")
            if val:
                return val
        except Exception:
            pass

        # 2. Try GTK 3.0 settings.ini
        try:
            ini_path = os.path.expanduser("~/.config/gtk-3.0/settings.ini")
            if os.path.isfile(ini_path):
                cfg = configparser.ConfigParser(interpolation=None)
                cfg.read(ini_path)
                if "Settings" in cfg and "gtk-icon-theme-name" in cfg["Settings"]:
                    val = cfg["Settings"]["gtk-icon-theme-name"].strip().strip("'\"")
                    if val:
                        return val
        except Exception:
            pass

        return ""

    @classmethod
    def _init_gtk(cls):
        if not cls._gtk_initialized:
            try:
                import gi
                gi.require_version("Gtk", "3.0")
                from gi.repository import Gtk

                system_theme = cls.get_system_icon_theme_name()
                settings = Gtk.Settings.get_default()
                if system_theme and settings:
                    settings.set_property("gtk-icon-theme-name", system_theme)

                cls._gtk_theme = Gtk.IconTheme.get_default()
                user_home = os.path.expanduser("~")
                cls._gtk_theme.append_search_path(os.path.join(user_home, ".local/share/icons"))
                cls._gtk_theme.append_search_path(os.path.join(user_home, ".icons"))
            except Exception:
                cls._gtk_theme = None
            cls._gtk_initialized = True

    @classmethod
    def resolve_icon_path(cls, icon_name: str, size: int = 32) -> str:
        target = icon_name.strip() if icon_name else "application-x-executable"

        # If it's already an absolute path to a file that exists
        if os.path.isabs(target) and os.path.exists(target):
            return target

        cls._init_gtk()

        candidates = [target]

        # Specific fallbacks for star/favorite icon (collapsed category)
        if target in ("emblem-favorite", "starred", "star", "rating-rated", "favorite"):
            for s in ["favorite", "favorites", "emblem-favorite", "rating-rated", "starred", "star", "bookmark-new", "folder-favorites", "folder-starred", "start-here"]:
                if s not in candidates:
                    candidates.append(s)

        # Specific fallbacks for category icons
        category_fallbacks = {
            "folder-network": ["applications-internet", "network-workgroup", "preferences-system-network", "internet-web-browser"],
            "folder-code": ["applications-development", "preferences-desktop", "utilities-terminal"],
            "folder-games": ["applications-games", "input-gaming"],
            "folder-music": ["folder-sound", "applications-multimedia", "audio-x-generic"],
            "folder-pictures": ["applications-graphics", "image-x-generic"],
            "folder-documents": ["applications-office", "x-office-document"],
            "preferences-system": ["preferences-desktop", "system-run"],
            "folder-utils": ["applications-utilities", "utilities-terminal"],
            "system-shutdown": ["system-log-out", "application-exit"],
        }
        if target in category_fallbacks:
            for c in category_fallbacks[target]:
                if c not in candidates:
                    candidates.append(c)

        # Specific fallbacks for web app icons
        if any(target.startswith(prefix) for prefix in ("chrome-", "brave-", "msedge-", "ffpwa-", "webapp-")):
            for w in ["internet-web-browser", "google-chrome", "brave-browser", "browser", "text-html"]:
                if w not in candidates:
                    candidates.append(w)

        for g in ["application-x-executable", "system-run", "preferences-other", "exec", "utilities-terminal"]:
            if g not in candidates:
                candidates.append(g)

        # 1. Try GTK Icon Theme lookup
        if cls._gtk_theme:
            for cand in candidates:
                try:
                    info = cls._gtk_theme.lookup_icon(cand, size, 0)
                    if info:
                        filename = info.get_filename()
                        if filename and os.path.exists(filename):
                            return filename
                except Exception:
                    pass

        # 2. Fallback search paths in common system and user icon directories
        user_home = os.path.expanduser("~")
        base_dirs = [
            os.path.join(user_home, ".local/share/icons/hicolor"),
            os.path.join(user_home, ".local/share/icons"),
            os.path.join(user_home, ".icons"),
            "/usr/share/icons/Papirus-Dark",
            "/usr/share/icons/Papirus",
            "/usr/share/icons/Adwaita",
            "/usr/share/icons/hicolor",
            "/usr/share/pixmaps",
        ]
        sub_types = ["places", "actions", "categories", "emblems", "status", "preferences", "apps", "mimetypes", "devices"]
        sizes = ["32x32", "48x48", "scalable", "24x24", "16x16", "256x256", "512x512", "symbolic"]

        fallback_dirs = []
        for bd in base_dirs:
            fallback_dirs.append(bd)
            for sz in sizes:
                for st in sub_types:
                    fallback_dirs.append(os.path.join(bd, sz, st))
                    fallback_dirs.append(os.path.join(bd, st))

        for cand in candidates:
            for d in fallback_dirs:
                if not os.path.exists(d):
                    continue
                for ext in ["", ".png", ".svg", ".xpm"]:
                    candidate_file = os.path.join(d, cand + ext)
                    if os.path.exists(candidate_file):
                        return candidate_file

        return target

