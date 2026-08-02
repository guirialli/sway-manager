import unittest
from unittest.mock import MagicMock, patch, mock_open
from infrastructure.power.sysfs_battery_repository import SysfsBatteryRepository


class TestSysfsBatteryRepository(unittest.TestCase):
    def test_get_state_supported_conservation_on(self):
        repo = SysfsBatteryRepository()
        with patch.object(repo, "_find_control_file", return_value="/sys/path/conservation_mode"):
            with patch("builtins.open", mock_open(read_data="1")):
                state = repo.get_state()
                self.assertTrue(state.is_supported)
                self.assertTrue(state.is_conservation_on)
                self.assertEqual(state.raw_value, 1)

    def test_get_state_unsupported(self):
        repo = SysfsBatteryRepository()
        with patch.object(repo, "_find_control_file", return_value=None):
            state = repo.get_state()
            self.assertFalse(state.is_supported)
            self.assertFalse(state.is_conservation_on)

    def test_toggle_success_direct_write(self):
        mock_notify = MagicMock()
        repo = SysfsBatteryRepository(notification_repo=mock_notify)
        with patch.object(repo, "_find_control_file", return_value="/sys/path/conservation_mode"):
            m = mock_open(read_data="1")
            with patch("builtins.open", m):
                # When re-reading after write to verify, return "0"
                m.return_value.read.side_effect = ["1", "0"]
                ok, msg = repo.toggle()
                self.assertTrue(ok)
                self.assertIn("Conservação Desligada", msg)
                mock_notify.notify.assert_called_once()
                self.assertEqual(mock_notify.notify.call_args[0][0].urgency, "normal")

    def test_toggle_failure_returns_error_notification(self):
        mock_notify = MagicMock()
        repo = SysfsBatteryRepository(notification_repo=mock_notify)
        with patch.object(repo, "_find_control_file", return_value="/sys/path/conservation_mode"):
            with patch("builtins.open", side_effect=[mock_open(read_data="1").return_value, PermissionError("Denied")]):
                with patch("subprocess.run") as mock_sub:
                    mock_sub.return_value.returncode = 126
                    ok, msg = repo.toggle()
                    self.assertFalse(ok)
                    self.assertIn("ERROR:", msg)
                    mock_notify.notify.assert_called_once()
                    self.assertEqual(mock_notify.notify.call_args[0][0].urgency, "critical")


if __name__ == "__main__":
    unittest.main()
