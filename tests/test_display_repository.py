import unittest
from unittest.mock import MagicMock, patch
from domain.display.entities import DisplaySwitchType
from infrastructure.display.sway_display_repository import SwayDisplayRepository


class TestSwayDisplayRepository(unittest.TestCase):
    def setUp(self):
        self.repo = SwayDisplayRepository()

    @patch("subprocess.run")
    def test_get_connected_monitors_count(self, mock_run):
        mock_res = MagicMock()
        mock_res.returncode = 0
        mock_res.stdout = '[{"name": "eDP-1"}, {"name": "HDMI-A-1"}]'
        mock_run.return_value = mock_res

        count = self.repo.get_connected_monitors_count()
        self.assertEqual(count, 2)

    @patch("subprocess.run")
    def test_get_current_layout_pc_only(self, mock_run):
        def subprocess_side_effect(cmd, **kwargs):
            res = MagicMock()
            res.returncode = 0
            if "get_outputs" in str(cmd):
                res.stdout = '[{"name": "eDP-1", "active": true}, {"name": "HDMI-A-1", "active": false}]'
            elif "grep" in str(cmd):
                res.stdout = '"name": "HDMI-A-1"\n"name": "eDP-1"\n'
            else:
                res.stdout = ""
            return res

        mock_run.side_effect = subprocess_side_effect
        layout = self.repo.get_current_layout()
        self.assertEqual(layout, DisplaySwitchType.PC_ONLY)


if __name__ == "__main__":
    unittest.main()
