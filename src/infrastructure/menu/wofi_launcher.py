import os
import html
import subprocess
from typing import List, Optional, Dict
from domain.menu.entities import MenuItem
from domain.menu.repositories import IMenuRepository
from infrastructure.menu.desktop_parser import DesktopParser
from infrastructure.menu.icon_resolver import IconResolver


class WofiRepository(IMenuRepository):
    _cached_items: Optional[List[MenuItem]] = None
    _cached_line_keys: Dict[str, str] = {}

    def __init__(self, parser: Optional[DesktopParser] = None):
        self.parser = parser or DesktopParser()

    @classmethod
    def _generate_line_key(cls, item: MenuItem) -> str:
        escaped_name = html.escape(item.normalized_name)
        icon_path = IconResolver.resolve_icon_path(item.icon)
        if item.is_system_action:
            search_tag = f'<span alpha="1%">/s /sessao /sistema /s {escaped_name} /s{escaped_name}</span>'
            return f"img:{icon_path}:text:<b>{escaped_name}</b> {search_tag}"
        else:
            search_tag = f'<span alpha="1%">/a /app /programa /a {escaped_name} /a{escaped_name}</span>'
            return f"img:{icon_path}:text:<b>{escaped_name}</b> {search_tag}"

    @classmethod
    def preload_cache(cls) -> List[MenuItem]:
        parser = DesktopParser()
        items = parser.parse_all()
        cls._cached_items = items
        cls._cached_line_keys = {}
        for item in items:
            key = f"{item.normalized_name}::{item.exec_cmd}"
            cls._cached_line_keys[key] = cls._generate_line_key(item)
        return items

    def get_menu_items(self, category_filter: Optional[str] = None) -> List[MenuItem]:
        if WofiRepository._cached_items is not None:
            items = WofiRepository._cached_items
        else:
            items = self.parser.parse_all()

        if not category_filter:
            return items

        filter_str = category_filter.strip()
        filter_lower = filter_str.lower()

        # /a mode or /a query (e.g. "/a", "/a tes", "/ates")
        if filter_lower == "/a" or filter_lower.startswith("/a ") or (filter_lower.startswith("/a") and len(filter_lower) > 2 and filter_lower[2] != "p"):
            query = filter_str[2:].strip()
            if not query:
                return [i for i in items if not i.is_system_action]
            q_clean = self.parser.strip_accents(query)
            return [
                i for i in items
                if not i.is_system_action
                and q_clean in self.parser.strip_accents(i.normalized_name)
            ]

        # /s mode or /s query (e.g. "/s", "/s d", "/sd")
        if filter_lower == "/s" or filter_lower.startswith("/s ") or filter_lower.startswith("/s"):
            query = filter_str[2:].strip()
            if not query:
                return [i for i in items if i.is_system_action]
            q_clean = self.parser.strip_accents(query)
            return [
                i for i in items
                if i.is_system_action
                and q_clean in self.parser.strip_accents(i.normalized_name)
            ]

        # General search filter
        f_clean = self.parser.strip_accents(filter_lower)
        return [
            item for item in items
            if f_clean in self.parser.strip_accents(item.normalized_name)
        ]

    def launch_menu(self, items: List[MenuItem], prompt: str = "Buscar...") -> Optional[MenuItem]:
        if not items:
            return None

        line_to_item: Dict[str, MenuItem] = {}
        input_lines: List[str] = []

        for item in items:
            item_id = f"{item.normalized_name}::{item.exec_cmd}"
            line_key = WofiRepository._cached_line_keys.get(item_id)
            if not line_key:
                line_key = self._generate_line_key(item)

            input_lines.append(line_key)
            line_to_item[line_key] = item


        input_payload = "\n".join(input_lines)


        wofi_cmd = [
            "wofi",
            "--dmenu",
            "--allow-images",
            "--allow-markup",
            "--insensitive",
            "--no-actions",
            f"--prompt={prompt}",
        ]

        home_config = os.path.expanduser("~/.config/wofi/config")
        home_style = os.path.expanduser("~/.config/wofi/style.css")
        if os.path.exists(home_config):
            wofi_cmd.extend(["--conf", home_config])
        if os.path.exists(home_style):
            wofi_cmd.extend(["--style", home_style])

        try:
            process = subprocess.Popen(
                wofi_cmd,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            stdout, _ = process.communicate(input=input_payload)

            if process.returncode != 0 or not stdout:
                return None

            selected_line = stdout.strip()
            return line_to_item.get(selected_line)

        except Exception as ex:
            print(f"Erro ao executar Wofi: {ex}")
            return None

    def execute_item(self, item: MenuItem) -> None:
        if not item or not item.exec_cmd:
            return

        try:
            subprocess.Popen(
                item.exec_cmd,
                shell=True,
                start_new_session=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        except Exception as ex:
            print(f"Erro ao executar comando '{item.exec_cmd}': {ex}")
