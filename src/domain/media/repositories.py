from abc import ABC, abstractmethod
from domain.media.value_objects import ScreenshotMode


class IScreenshotRepository(ABC):
    @abstractmethod
    def take_screenshot(self, mode: ScreenshotMode) -> None:
        pass
