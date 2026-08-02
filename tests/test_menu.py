import os
import unittest
from unittest.mock import MagicMock
from domain.menu.entities import MenuItem
from infrastructure.menu.desktop_parser import DesktopParser
from application.menu.show_menu_use_case import ShowMenuUseCase


class TestMenu(unittest.TestCase):
    def test_strip_accents_and_sort(self):
        self.assertEqual(DesktopParser.strip_accents("Água"), "agua")
        self.assertEqual(DesktopParser.strip_accents("Código"), "codigo")

        items = ["Água", "América", "Brave", "Código", "Árvore"]
        sorted_items = sorted(items, key=DesktopParser.strip_accents)
        expected = ["Água", "América", "Árvore", "Brave", "Código"]
        self.assertEqual(sorted_items, expected)

    def test_sanitize_text(self):
        self.assertEqual(DesktopParser.sanitize_text("System > Terminal"), "Terminal")
        self.assertEqual(DesktopParser.sanitize_text("App <SubName>"), "App SubName")
        self.assertEqual(DesktopParser.sanitize_text("Normal App"), "Normal App")

    def test_normalize_name(self):
        self.assertEqual(DesktopParser.normalize_name("Firefox Web Browser"), "Firefox")
        self.assertEqual(
            DesktopParser.normalize_name("Visual Studio Code - Open Source"), "VS Code"
        )
        self.assertEqual(
            DesktopParser.normalize_name("GNU Image Manipulation Program"), "GIMP"
        )
        self.assertEqual(
            DesktopParser.normalize_name("Google Chrome (Web Browser)"), "Google Chrome"
        )

    def test_clean_exec_cmd(self):
        self.assertEqual(DesktopParser.clean_exec_cmd("firefox %u"), "firefox")
        self.assertEqual(DesktopParser.clean_exec_cmd("code --new-window %F"), "code --new-window")

    def test_map_category(self):
        self.assertEqual(DesktopParser.map_category("Network;WebBrowser;"), "Internet")
        self.assertEqual(DesktopParser.map_category("Development;IDE;"), "Desenvolvimento")
        self.assertEqual(DesktopParser.map_category("Game;ActionGame;"), "Jogos")

    def test_system_actions(self):
        actions = DesktopParser.get_system_actions()
        self.assertTrue(len(actions) >= 5)
        names = [a.normalized_name for a in actions]
        self.assertIn("Bloquear Tela", names)
        self.assertIn("Encerrar Sessão", names)

    def test_show_menu_use_case_execution(self):
        mock_repo = MagicMock()
        app_item = MenuItem(
            name="Firefox",
            normalized_name="Firefox",
            exec_cmd="firefox",
            icon="firefox",
            category="Internet",
        )

        mock_repo.get_menu_items.return_value = [app_item]
        mock_repo.launch_menu.return_value = app_item

        use_case = ShowMenuUseCase(mock_repo)
        success = use_case.execute()

        self.assertTrue(success)
        mock_repo.execute_item.assert_called_once_with(app_item)

    def test_icon_resolver(self):
        from infrastructure.menu.icon_resolver import IconResolver
        res = IconResolver.resolve_icon_path("firefox")
        self.assertTrue(bool(res))

    def test_generic_icon_fallback(self):
        from infrastructure.menu.icon_resolver import IconResolver
        # Test empty icon string fallback
        res_empty = IconResolver.resolve_icon_path("")
        self.assertTrue(os.path.isabs(res_empty))
        # Test non-existent icon string fallback
        res_invalid = IconResolver.resolve_icon_path("non-existent-app-icon-xyz-123")
        self.assertTrue(os.path.isabs(res_invalid))

    def test_chrome_webapp_icon_resolution(self):
        from infrastructure.menu.icon_resolver import IconResolver
        res = IconResolver.resolve_icon_path("chrome-kajebgjangihfbkjfejcanhanjmmbcfd-Default")
        self.assertTrue(os.path.isabs(res))

    def test_xfce_and_de_filtering(self):
        items = DesktopParser.parse_all()
        item_names = [i.normalized_name.lower() for i in items]
        self.assertNotIn("about xfce", item_names)
        self.assertNotIn("panel", item_names)
        self.assertNotIn("window manager", item_names)

    def test_system_actions_at_bottom(self):
        items = DesktopParser.parse_all()
        self.assertTrue(len(items) > 0)
        bottom_item = items[-1]
        self.assertTrue(bottom_item.is_system_action)

    def test_web_app_detection(self):
        self.assertTrue(DesktopParser.is_web_app({}, "google-chrome --app=https://youtube.com"))
        self.assertTrue(DesktopParser.is_web_app({}, "brave --app-id=12345"))
        self.assertTrue(DesktopParser.is_web_app({"Icon": "chrome-12345-Default"}, "google-chrome"))
        self.assertTrue(DesktopParser.is_web_app({}, "firefox", "chrome-youtube.desktop"))
        self.assertFalse(DesktopParser.is_web_app({}, "firefox"))

    def test_wofi_repository_prefix_filters(self):
        from infrastructure.menu.wofi_launcher import WofiRepository
        repo = WofiRepository()

        # /a filter returns only apps
        a_items = repo.get_menu_items("/a")
        self.assertTrue(all(not i.is_system_action for i in a_items))

        # /s filter returns only session items
        s_items = repo.get_menu_items("/s")
        self.assertTrue(all(i.is_system_action for i in s_items))

        # /a query filter returns matching app
        a_query = repo.get_menu_items("/a Adv")
        self.assertTrue(len(a_query) > 0)
        self.assertTrue(all(not i.is_system_action for i in a_query))

        # /s Des filter returns Desligar
        s_des = repo.get_menu_items("/s Des")
        self.assertTrue(any(i.normalized_name == "Desligar" for i in s_des))


if __name__ == "__main__":
    unittest.main()



