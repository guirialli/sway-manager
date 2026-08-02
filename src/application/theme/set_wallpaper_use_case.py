from typing import Optional
from domain.theme.repositories import IWallpaperRepository


class SetWallpaperUseCase:
    def __init__(self, repository: IWallpaperRepository):
        self.repository = repository

    def execute(self, image_path: str) -> None:
        self.repository.set_wallpaper(image_path)

    def get_current(self) -> Optional[str]:
        return self.repository.get_current_wallpaper()

    def get_wallpaper_folder(self) -> str:
        return self.repository.get_wallpaper_folder()

    def set_wallpaper_folder(self, folder_path: str) -> None:
        self.repository.set_wallpaper_folder(folder_path)

