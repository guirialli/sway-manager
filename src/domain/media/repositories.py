from abc import ABC, abstractmethod
from domain.media.value_objects import ScreenshotMode


class IScreenshotRepository(ABC):
    @abstractmethod
    def take_screenshot(self, mode: ScreenshotMode) -> None:
        pass

    @abstractmethod
    def get_screenshot_folder(self) -> str:
        pass

    @abstractmethod
    def set_screenshot_folder(self, folder_path: str) -> None:
        pass
