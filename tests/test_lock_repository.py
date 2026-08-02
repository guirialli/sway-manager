import unittest
from unittest.mock import MagicMock, patch
from application.power.lock_screen_use_case import LockScreenUseCase
from infrastructure.power.swaylock_repository import SwayLockRepository


class TestLockRepositoryAndUseCase(unittest.TestCase):
    def test_lock_screen_use_case(self):
        mock_repo = MagicMock()
        use_case = LockScreenUseCase(mock_repo)
        use_case.execute()
        mock_repo.lock_screen.assert_called_once()

    @patch("subprocess.run")
    @patch("subprocess.Popen")
    @patch("shutil.which")
    def test_swaylock_repository_dark_theme(self, mock_which, mock_popen, mock_run):
        mock_which.side_effect = lambda cmd: f"/usr/bin/{cmd}"
        mock_run.returncode = 1  # pgrep swaylock returns 1 (not running)

        mock_wp_repo = MagicMock()
        mock_wp_repo.get_current_wallpaper.return_value = "/tmp/test_wallpaper.jpg"

        mock_theme_repo = MagicMock()
        mock_theme_state = MagicMock()
        mock_theme_state.current_theme = "dark"
        mock_theme_repo.get_state.return_value = mock_theme_state

        repo = SwayLockRepository(wallpaper_repo=mock_wp_repo, theme_repo=mock_theme_repo)

        with patch("os.path.isfile", return_value=True):
            repo.lock_screen()

        mock_popen.assert_called_once()
        idle_cmd = mock_popen.call_args[0][0]
        self.assertIn("swayidle", idle_cmd)
        self.assertIn("60", idle_cmd)

        self.assertTrue(mock_run.called)
        swaylock_calls = [call for call in mock_run.call_args_list if call[0][0][0] == "/usr/bin/swaylock"]
        self.assertEqual(len(swaylock_calls), 1)
        args = swaylock_calls[0][0][0]
        self.assertIn("-i", args)
        self.assertIn("/tmp/test_wallpaper.jpg", args)
        self.assertIn("1a0a28", args)


if __name__ == "__main__":
    unittest.main()
