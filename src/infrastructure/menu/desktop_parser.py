import os
import re
import html
import configparser
import unicodedata
from typing import List, Optional
from domain.menu.entities import MenuItem


KNOWN_NAME_MAP = {
    "firefox web browser": "Firefox",
    "visual studio code - open source": "VS Code",
    "code - oss": "VS Code",
    "google chrome (web browser)": "Google Chrome",
    "gnu image manipulation program": "GIMP",
    "vlc media player": "VLC",
    "brave web browser": "Brave",
    "org.gnome.nautilus": "Arquivos",
    "gnome-terminal": "Terminal",
}

CATEGORY_MAP = [
    ({"webbrowser", "network"}, "Internet"),
    ({"development", "ide", "building"}, "Desenvolvimento"),
    ({"game"}, "Jogos"),
    ({"audiovideo", "audio", "video", "player"}, "Multimídia"),
    ({"graphics", "2dgraphics", "vectorgraphics", "rastergraphics"}, "Gráficos"),
    ({"office", "wordprocessor", "spreadsheet", "presentation"}, "Escritório"),
    ({"settings", "desktopsettings", "hardwaresettings"}, "Configurações"),
    ({"utility", "system", "terminalemulator", "filemanager"}, "Utilitários"),
]

EXPLICIT_IGNORE_EXECS = {"xfce4-about", "xfce4-appfinder", "xfce4-session-logout", "xfce4-panel", "exo-open"}
EXPLICIT_IGNORE_PREFIXES = ("xfce4-settings", "xfwm4", "xfce-settings", "xfce-session", "xfce-wm", "xfce-backdrop", "panel-")


class DesktopParser:
    @staticmethod
    def get_search_directories() -> List[str]:
        home = os.path.expanduser("~")
        return [
            os.path.join(home, ".local/share/applications"),
            "/usr/local/share/applications",
            "/usr/share/applications",
            "/var/lib/snap/desktop/applications",
            os.path.join(home, ".local/share/flatpak/exports/share/applications"),
            "/var/lib/flatpak/exports/share/applications",
        ]

    @classmethod
    def clean_exec_cmd(cls, exec_cmd: str) -> str:
        if not exec_cmd:
            return ""
        # Strip desktop entry field codes like %u, %U, %f, %F, %k, %i, %c, %b
        cleaned = re.sub(r"%\w", "", exec_cmd)
        return cleaned.strip()

    @classmethod
    def strip_accents(cls, text: str) -> str:
        if not text:
            return ""
        normalized = unicodedata.normalize("NFKD", text)
        return "".join(c for c in normalized if not unicodedata.combining(c)).lower()

    @classmethod
    def sanitize_text(cls, text: str) -> str:
        if not text:
            return ""
        cleaned = text
        if ">" in cleaned:
            parts = [p.strip() for p in cleaned.split(">") if p.strip()]
            if parts:
                cleaned = parts[-1]
        cleaned = re.sub(r"[<>]", "", cleaned)
        return cleaned.strip()

    @classmethod
    def normalize_name(cls, name: str, generic_name: Optional[str] = None) -> str:
        if not name:
            return "Aplicativo"

        cleaned = cls.sanitize_text(name)
        lower_name = cleaned.lower().strip()
        if lower_name in KNOWN_NAME_MAP:
            return KNOWN_NAME_MAP[lower_name]

        # Strip redundant trailing phrases
        cleaned = re.sub(
            r"\s*(\(Web Browser\)|Web Browser|GNU Image Manipulation Program|- Open Source|Community Edition|\(PNG only\))\s*$",
            "",
            cleaned,
            flags=re.IGNORECASE,
        ).strip()

        # If name contains GenericName (e.g. "Firefox Web Browser"), strip generic part
        if generic_name and generic_name.lower() in cleaned.lower():
            pattern = re.escape(generic_name)
            cleaned = re.sub(pattern, "", cleaned, flags=re.IGNORECASE).strip()

        return cleaned if cleaned else name

    @classmethod
    def map_category(cls, categories_str: str) -> str:
        if not categories_str:
            return "Aplicativos"

        cats = {c.lower() for c in categories_str.split(";") if c.strip()}
        for keys, display_cat in CATEGORY_MAP:
            if cats.intersection(keys):
                return display_cat

        return "Aplicativos"

    @classmethod
    def get_category_headers(cls) -> List[MenuItem]:
        categories = [
            ("Internet", "folder-network"),
            ("Desenvolvimento", "folder-code"),
            ("Jogos", "folder-games"),
            ("Multimídia", "folder-music"),
            ("Gráficos", "folder-pictures"),
            ("Escritório", "folder-documents"),
            ("Configurações", "preferences-system"),
            ("Utilitários", "folder-utils"),
            ("Sistema / Sair", "system-shutdown"),
        ]
        return [
            MenuItem(
                name=display_cat,
                normalized_name=display_cat,
                exec_cmd="",
                icon=icon,
                category="Categorias",
                comment=f"Filtrar apenas aplicativos da categoria {display_cat}",
                is_category_header=True,
                category_target=display_cat,
            )
            for display_cat, icon in categories
        ]

    @classmethod
    def get_system_actions(cls) -> List[MenuItem]:
        return [
            MenuItem(
                name="Bloquear Tela",
                normalized_name="Bloquear Tela",
                exec_cmd="swaylock -f -c 000000",
                icon="system-lock-screen",
                category="Sistema / Sair",
                comment="Bloqueia a sessão atual do Sway",
                is_system_action=True,
            ),
            MenuItem(
                name="Encerrar Sessão",
                normalized_name="Encerrar Sessão",
                exec_cmd="swaymsg exit",
                icon="system-log-out",
                category="Sistema / Sair",
                comment="Encerra o gerenciador de janelas Sway",
                is_system_action=True,
            ),
            MenuItem(
                name="Suspender",
                normalized_name="Suspender",
                exec_cmd="systemctl suspend",
                icon="system-suspend",
                category="Sistema / Sair",
                comment="Coloca o computador em modo de suspensão",
                is_system_action=True,
            ),
            MenuItem(
                name="Reiniciar",
                normalized_name="Reiniciar",
                exec_cmd="systemctl reboot",
                icon="system-reboot",
                category="Sistema / Sair",
                comment="Reinicia o sistema operacional",
                is_system_action=True,
            ),
            MenuItem(
                name="Desligar",
                normalized_name="Desligar",
                exec_cmd="systemctl poweroff",
                icon="system-shutdown",
                category="Sistema / Sair",
                comment="Desliga o computador",
                is_system_action=True,
            ),
        ]

    @classmethod
    def get_collapsed_category_header(cls) -> MenuItem:
        return MenuItem(
            name="Categorias",
            normalized_name="Categorias",
            exec_cmd="",
            icon="emblem-favorite",
            category="Categorias",
            comment="Expandir para ver todas as categorias de aplicativos",
            is_category_header=True,
            category_target="CATEGORIES_VIEW",
        )

    @classmethod
    def is_web_app(cls, section, clean_exec: str, desktop_filename: str = "") -> bool:
        exec_lower = clean_exec.lower()
        fn_lower = desktop_filename.lower()

        icon_val = section.get("Icon", "") if hasattr(section, "get") else ""
        icon_lower = str(icon_val).lower() if icon_val else ""

        cats_val = section.get("Categories", "") if hasattr(section, "get") else ""
        cats_lower = str(cats_val).lower() if cats_val else ""

        if "--app=" in exec_lower or "--app-id=" in exec_lower or "webapp" in exec_lower:
            return True
        if any(prefix in fn_lower for prefix in ("chrome-", "brave-", "msedge-", "ffpwa-", "webapp-")):
            return True
        if any(prefix in icon_lower for prefix in ("chrome-", "brave-", "msedge-", "ffpwa-", "webapp-")):
            return True
        if "webapp" in cats_lower or "x-webapp" in cats_lower:
            return True
        return False


    @classmethod
    def parse_all(cls) -> List[MenuItem]:
        items: List[MenuItem] = []
        seen_execs = set()

        for d in cls.get_search_directories():
            if not os.path.exists(d):
                continue
            for root, _, files in os.walk(d):
                for f in files:
                    if not f.endswith(".desktop"):
                        continue
                    file_path = os.path.join(root, f)
                    try:
                        parser = configparser.ConfigParser(interpolation=None, strict=False)
                        parser.read(file_path, encoding="utf-8")
                        if not parser.has_section("Desktop Entry"):
                            continue
                        section = parser["Desktop Entry"]

                        # Skip hidden / nodisplay
                        if section.getboolean("NoDisplay", fallback=False) or section.getboolean("Hidden", fallback=False):
                            continue

                        # Check NotShowIn
                        not_show_str = section.get("NotShowIn", fallback="")
                        if not_show_str:
                            not_show_envs = [e.strip().lower() for e in not_show_str.split(";") if e.strip()]
                            if "sway" in not_show_envs:
                                continue

                        # Check OnlyShowIn
                        only_show_str = section.get("OnlyShowIn", fallback="")
                        if only_show_str:
                            only_show_envs = [e.strip().lower() for e in only_show_str.split(";") if e.strip()]
                            if "sway" not in only_show_envs and "wayland" not in only_show_envs:
                                continue

                        raw_exec = section.get("Exec", fallback="")
                        if not raw_exec:
                            continue

                        clean_exec = cls.clean_exec_cmd(raw_exec)
                        if clean_exec in seen_execs:
                            continue

                        # Check explicit XFCE DE-only blacklist
                        f_lower = f.lower()
                        e_lower = clean_exec.lower()
                        if any(f_lower.startswith(prefix) for prefix in EXPLICIT_IGNORE_PREFIXES) or \
                           any(e_lower.startswith(cmd) for cmd in EXPLICIT_IGNORE_EXECS):
                            continue

                        seen_execs.add(clean_exec)

                        raw_name = section.get("Name", fallback="").strip()
                        if not raw_name:
                            continue

                        generic_name = section.get("GenericName", fallback="")
                        normalized_name = cls.normalize_name(raw_name, generic_name)

                        icon = section.get("Icon", fallback="application-x-executable")
                        categories = section.get("Categories", fallback="")
                        category = cls.map_category(categories)
                        comment = cls.sanitize_text(section.get("Comment", fallback=""))

                        web_app = cls.is_web_app(section, clean_exec, f)
                        if web_app and category == "Aplicativos":
                            category = "Internet"


                        items.append(
                            MenuItem(
                                name=raw_name,
                                normalized_name=normalized_name,
                                exec_cmd=clean_exec,
                                icon=icon,
                                category=category,
                                comment=comment,
                                is_system_action=False,
                                is_web_app=web_app,
                            )
                        )
                    except Exception:
                        continue

        # Sort applications strictly alphabetically (A-Z) stripping accents
        items.sort(key=lambda x: cls.strip_accents(x.normalized_name))

        # 2. System actions at the very end
        system_actions = cls.get_system_actions()
        items.extend(system_actions)

        return items


