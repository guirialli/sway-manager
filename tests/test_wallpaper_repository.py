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

    def test_set_and_get_wallpaper_folder(self):
        # SWAY_MANAGER_TEST_MODE=1 garante que a gravação vá para /tmp descartável
        self.use_case.set_wallpaper_folder(self.test_dir)
        retrieved_folder = self.use_case.get_wallpaper_folder()
        self.assertEqual(retrieved_folder, self.test_dir)

    def test_get_wallpaper_folder_default_fallback(self):
        folder = self.repo.get_wallpaper_folder()
        self.assertTrue(os.path.isdir(folder))


if __name__ == "__main__":
    unittest.main()
