import os
import tempfile
import shutil
import unittest
from unittest.mock import MagicMock, patch
from domain.clipboard.entities import ClipboardItem
from application.clipboard.manage_clipboard_use_case import ManageClipboardUseCase
from infrastructure.clipboard.cliphist_repository import CliphistRepository


class TestClipboardDomainAndUseCase(unittest.TestCase):
    def setUp(self):
        self.mock_repo = MagicMock()

    def test_clipboard_item_entity(self):
        item = ClipboardItem(
            id="1",
            text="Hello World",
            raw_preview="Hello World",
            is_image=False,
            is_favorite=True,
        )
        self.assertEqual(item.id, "1")
        self.assertEqual(item.text, "Hello World")
        self.assertTrue(item.is_favorite)
        self.assertFalse(item.is_action)

    def test_use_case_execute_copy(self):
        item = ClipboardItem(
            id="1", text="Sample", raw_preview="Sample", is_image=False
        )
        self.mock_repo.get_clipboard_items.return_value = [item]
        self.mock_repo.launch_menu.return_value = item

        use_case = ManageClipboardUseCase(self.mock_repo)
        result = use_case.execute()

        self.assertTrue(result)
        self.mock_repo.ensure_daemon_running.assert_called_once()
        self.mock_repo.copy_to_clipboard.assert_called_once_with(item)

    def test_use_case_execute_clear_action(self):
        use_case = ManageClipboardUseCase(self.mock_repo)
        self.mock_repo.clear_history.return_value = True

        result = use_case.execute(action="clear")

        self.assertTrue(result)
        self.mock_repo.clear_history.assert_called_once()

    def test_use_case_select_clear_action_item(self):
        action_item = ClipboardItem(
            id="action_clear",
            text="🧹 [Limpar Histórico]",
            raw_preview="🧹 Limpar Histórico",
            is_action=True,
            action_type="clear",
        )
        self.mock_repo.get_clipboard_items.return_value = [action_item]
        self.mock_repo.launch_menu.return_value = action_item
        self.mock_repo.clear_history.return_value = True

        use_case = ManageClipboardUseCase(self.mock_repo)
        result = use_case.execute()

        self.assertTrue(result)
        self.mock_repo.clear_history.assert_called_once()

    def test_use_case_pin_favorite(self):
        history_item = ClipboardItem(
            id="10", text="Item para fixar", raw_preview="Item para fixar"
        )
        action_pin_item = ClipboardItem(
            id="10",
            text="📌 [Fixar nos Favoritos] Item para fixar",
            raw_preview="Item para fixar",
            is_action=True,
            action_type="pin",
        )

        self.mock_repo.get_clipboard_items.return_value = [history_item]
        self.mock_repo.launch_menu.return_value = action_pin_item
        self.mock_repo.add_favorite.return_value = True

        use_case = ManageClipboardUseCase(self.mock_repo)
        result = use_case.execute(action="pin")

        self.assertTrue(result)
        self.mock_repo.add_favorite.assert_called_once()


class TestCliphistRepository(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.mkdtemp()
        self.fav_json = os.path.join(self.test_dir, "favorites.json")
        self.cache_dir = os.path.join(self.test_dir, "thumbs")
        self.repo = CliphistRepository(
            fav_config_path=self.fav_json,
            cache_dir=self.cache_dir,
        )

    def tearDown(self):
        shutil.rmtree(self.test_dir)

    def test_save_and_get_favorites(self):
        item = ClipboardItem(
            id="fav_1",
            text="Texto Favorito",
            raw_preview="Texto Favorito",
            is_favorite=True,
        )
        self.repo.save_favorites([item])

        favs = self.repo.get_favorites()
        self.assertEqual(len(favs), 1)
        self.assertEqual(favs[0].id, "fav_1")
        self.assertEqual(favs[0].text, "Texto Favorito")

    def test_add_and_remove_favorite(self):
        item = ClipboardItem(
            id="123",
            text="Novo Favorito",
            raw_preview="Novo Favorito",
        )
        with patch.object(self.repo, "_decode_cliphist_item", return_value=b"Novo Favorito"):
            self.repo.add_favorite(item)

        favs = self.repo.get_favorites()
        self.assertEqual(len(favs), 1)
        fav_id = favs[0].id

        self.repo.remove_favorite(fav_id)
        favs_after = self.repo.get_favorites()
        self.assertEqual(len(favs_after), 0)

    @patch("shutil.which")
    @patch("subprocess.run")
    def test_get_clipboard_items_parsing(self, mock_run, mock_which):
        mock_which.return_value = "/usr/bin/cliphist"
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout="100\tPrimeiro texto do clipboard\n101\t[[ binary data 1920x1080 png 1 MiB ]]\n",
        )

        with patch.object(self.repo, "_get_or_create_thumbnail", return_value="/tmp/thumb.png"):
            items = self.repo.get_clipboard_items()

        # Deve incluir 2 Ações Especiais + 2 itens do cliphist
        self.assertGreaterEqual(len(items), 4)
        self.assertEqual(items[0].action_type, "clear")
        self.assertEqual(items[1].action_type, "manage_favorites")
        self.assertEqual(items[2].id, "100")
        self.assertEqual(items[2].text, "Primeiro texto do clipboard")
        self.assertEqual(items[3].id, "101")
        self.assertTrue(items[3].is_image)
        self.assertEqual(items[3].image_path, "/tmp/thumb.png")

    @patch("subprocess.Popen")
    @patch("os.path.exists")
    def test_launch_menu(self, mock_exists, mock_popen):
        mock_exists.return_value = False
        mock_proc = MagicMock()
        mock_proc.communicate.side_effect = lambda input=None: (input.strip() if input else "", "")
        mock_proc.returncode = 0
        mock_popen.return_value = mock_proc

        item = ClipboardItem(
            id="1",
            text="Teste",
            raw_preview="Teste",
        )
        selected = self.repo.launch_menu([item], prompt="Test Prompt...")

        self.assertIsNotNone(selected)
        self.assertEqual(selected.id, "1")
        self.assertEqual(selected.text, "Teste")


if __name__ == "__main__":
    unittest.main()
