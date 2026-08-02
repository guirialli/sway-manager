import os
import unittest
from unittest.mock import MagicMock, patch
from domain.media.value_objects import ScreenshotMode
from infrastructure.media.grim_slurp_screenshot_repository import (
    GrimSlurpScreenshotRepository,
)


class TestGrimSlurpScreenshotRepository(unittest.TestCase):
    def setUp(self):
        self.notification_repo = MagicMock()
        self.repo = GrimSlurpScreenshotRepository(
            notification_repo=self.notification_repo
        )

    def test_parse_geometry_valid(self):
        geom_str = "100,200 800x600\n"
        result = self.repo._parse_geometry(geom_str)
        self.assertEqual(result, (100, 200, 800, 600))

    def test_parse_geometry_invalid(self):
        self.assertIsNone(self.repo._parse_geometry("invalid geometry"))
        self.assertIsNone(self.repo._parse_geometry(""))

    def test_find_focused_rect(self):
        tree = {
            "nodes": [
                {
                    "focused": False,
                    "nodes": [
                        {
                            "focused": True,
                            "rect": {"x": 50, "y": 60, "width": 500, "height": 400},
                        }
                    ],
                }
            ]
        }
        rect = self.repo._find_focused_rect(tree)
        self.assertEqual(rect, (50, 60, 500, 400))

    def test_find_focused_rect_not_found(self):
        tree = {"nodes": [{"focused": False}]}
        self.assertIsNone(self.repo._find_focused_rect(tree))

    @patch("presentation.gui.widgets.freeze_selection_overlay.FreezeSelectionOverlay.select_area")
    @patch("subprocess.run")
    @patch("os.path.isfile")
    def test_take_screenshot_area_success(
        self, mock_isfile, mock_subprocess, mock_select_area
    ):
        mock_isfile.return_value = True
        mock_select_area.return_value = True

        def subprocess_side_effect(cmd, **kwargs):
            res = MagicMock()
            if isinstance(cmd, list) and cmd[0] == "grim":
                res.returncode = 0
            else:
                res.returncode = 0
                res.stdout = "[]"
            return res

        mock_subprocess.side_effect = subprocess_side_effect

        self.repo.take_screenshot(ScreenshotMode.AREA)
        mock_select_area.assert_called_once()
        self.notification_repo.notify.assert_called_once()

    @patch("presentation.gui.widgets.freeze_selection_overlay.FreezeSelectionOverlay.select_area")
    @patch("subprocess.run")
    @patch("os.path.isfile")
    def test_take_screenshot_area_cancelled(self, mock_isfile, mock_subprocess, mock_select_area):
        mock_isfile.return_value = True
        mock_select_area.return_value = False

        def subprocess_side_effect(cmd, **kwargs):
            res = MagicMock()
            if isinstance(cmd, list) and cmd[0] == "grim":
                res.returncode = 0
            else:
                res.returncode = 0
                res.stdout = "[]"
            return res

        mock_subprocess.side_effect = subprocess_side_effect

        self.repo.take_screenshot(ScreenshotMode.AREA)
        self.notification_repo.notify.assert_not_called()

    def test_set_and_get_screenshot_folder(self):
        import tempfile
        import shutil

        temp_dir = tempfile.mkdtemp()
        try:
            self.repo.set_screenshot_folder(temp_dir)
            folder = self.repo.get_screenshot_folder()
            self.assertEqual(folder, temp_dir)
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)


if __name__ == "__main__":
    unittest.main()
