from domain.media.value_objects import ScreenshotMode
from domain.media.repositories import IScreenshotRepository


class TakeScreenshotUseCase:
    def __init__(self, repository: IScreenshotRepository):
        self.repository = repository

    def execute(self, mode: ScreenshotMode) -> None:
        self.repository.take_screenshot(mode)
