import os
import tempfile
import unittest
from unittest.mock import patch, mock_open
from domain.theme.entities import LightDMSettings
from infrastructure.theme.lightdm_repository import LightDMRepository


class TestLightDMRepository(unittest.TestCase):
    def setUp(self):
        self.repo = LightDMRepository()

    def test_get_available_users_filters_nobody(self):
        passwd_content = (
            "root:x:0:0:root:/root:/bin/bash\n"
            "daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n"
            "guilherme:x:1000:1000:Guilherme,,,:/home/guilherme:/bin/bash\n"
            "nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin\n"
        )
        with patch("builtins.open", mock_open(read_data=passwd_content)):
            with patch("getpass.getuser", return_value="guilherme"):
                users = self.repo.get_available_users()
                self.assertIn("guilherme", users)
                self.assertNotIn("nobody", users)
                self.assertNotIn("root", users)
                self.assertNotIn("daemon", users)

    def test_get_available_sessions(self):
        with patch("os.path.isdir", return_value=True):
            with patch("os.listdir", side_effect=[["sway.desktop", "xfce.desktop"], ["lxqt.desktop"]]):
                sessions = self.repo.get_available_sessions()
                self.assertIn("sway", sessions)
                self.assertIn("xfce", sessions)
                self.assertIn("lxqt", sessions)
                self.assertEqual(sessions[0], "sway")

    def test_get_settings_from_temp_files(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            greeter_file = os.path.join(tmp_dir, "lightdm-gtk-greeter.conf")
            daemon_file = os.path.join(tmp_dir, "lightdm.conf")

            with open(greeter_file, "w") as f:
                f.write("[greeter]\ntheme-name = TestTheme\nclock-format = %H:%M\n")

            with open(daemon_file, "w") as f:
                f.write("[Seat:*]\nautologin-user = testuser\nuser-session = sway\n")

            self.repo.GREETER_CONFIG = greeter_file
            self.repo.DAEMON_CONFIG = daemon_file

            settings = self.repo.get_settings()
            self.assertEqual(settings.theme_name, "TestTheme")
            self.assertEqual(settings.clock_format, "%H:%M")
            self.assertEqual(settings.autologin_user, "testuser")
            self.assertEqual(settings.user_session, "sway")


if __name__ == "__main__":
    unittest.main()
