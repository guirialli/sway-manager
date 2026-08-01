from abc import ABC, abstractmethod
from typing import Optional
from domain.theme.entities import (
    ThemeState,
    LightDMSettings,
    AppearanceSettings,
    AvailableAppearanceOptions,
)


class IThemeRepository(ABC):
    @abstractmethod
    def get_state(self) -> ThemeState:
        pass

    @abstractmethod
    def toggle(self) -> str:
        pass

    @abstractmethod
    def get_appearance_settings(self) -> AppearanceSettings:
        pass

    @abstractmethod
    def get_available_options(self) -> AvailableAppearanceOptions:
        pass

    @abstractmethod
    def apply_appearance(self, settings: AppearanceSettings) -> bool:
        pass



class IWallpaperRepository(ABC):
    @abstractmethod
    def set_wallpaper(self, image_path: str) -> None:
        pass

    @abstractmethod
    def get_current_wallpaper(self) -> Optional[str]:
        pass


class ILightDMRepository(ABC):
    @abstractmethod
    def get_settings(self) -> LightDMSettings:
        pass

    @abstractmethod
    def save_settings(self, settings: LightDMSettings, image_source_path: Optional[str] = None) -> bool:
        pass
