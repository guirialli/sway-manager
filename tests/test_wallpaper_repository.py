import os
import shutil
import tempfile
import unittest
from infrastructure.theme.sway_wallpaper_repository import SwayWallpaperRepository
from application.theme.set_wallpaper_use_case import SetWallpaperUseCase


class TestSwayWallpaperRepositoryFolder(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.mkdtemp()
        self.repo = SwayWallpaperRepository()
        self.use_case = SetWallpaperUseCase(self.repo)

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)
        config_folder_file = os.path.expanduser("~/.config/sway/wallpaper_folder")
        if os.path.isfile(config_folder_file):
            try:
                os.remove(config_folder_file)
            except Exception:
                pass

    def test_set_and_get_wallpaper_folder(self):
        # Test saving a custom folder path
        self.use_case.set_wallpaper_folder(self.test_dir)
        retrieved_folder = self.use_case.get_wallpaper_folder()
        self.assertEqual(retrieved_folder, self.test_dir)

    def test_get_wallpaper_folder_default_fallback(self):
        # When no config exists, get_wallpaper_folder returns a valid existing directory
        folder = self.repo.get_wallpaper_folder()
        self.assertTrue(os.path.isdir(folder))


if __name__ == "__main__":
    unittest.main()
