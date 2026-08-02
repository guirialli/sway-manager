import os
import shutil
import tempfile
import unittest
from infrastructure.config.json_config_repository import JsonConfigRepository
from infrastructure.theme.sway_wallpaper_repository import SwayWallpaperRepository


class TestJsonConfigRepository(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.mkdtemp()
        self.config_path = os.path.join(self.test_dir, "config.json")
        self.repo = JsonConfigRepository(config_path=self.config_path)

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def test_get_and_set_setting(self):
        self.assertIsNone(self.repo.get_setting("non_existent_key"))
        self.assertEqual(
            self.repo.get_setting("non_existent_key", default="fallback"), "fallback"
        )

        ok = self.repo.set_setting("wallpaper_folder", "/tmp/wallpapers")
        self.assertTrue(ok)
        self.assertTrue(os.path.isfile(self.config_path))

        val = self.repo.get_setting("wallpaper_folder")
        self.assertEqual(val, "/tmp/wallpapers")

    def test_multiple_keys_preservation(self):
        self.repo.set_setting("key1", "val1")
        self.repo.set_setting("key2", 123)

        self.assertEqual(self.repo.get_setting("key1"), "val1")
        self.assertEqual(self.repo.get_setting("key2"), 123)


if __name__ == "__main__":
    unittest.main()
