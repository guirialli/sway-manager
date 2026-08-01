from abc import ABC, abstractmethod
from domain.display.entities import MonitoresSway, DisplaySwitchType


class IDisplayRepository(ABC):
    @abstractmethod
    def get_monitors(self) -> MonitoresSway:
        pass

    @abstractmethod
    def apply_config(self, mode: DisplaySwitchType) -> None:
        pass

    @abstractmethod
    def recarregar_sway(self) -> None:
        pass


class IBrightnessRepository(ABC):
    @abstractmethod
    def get_current_percentage(self) -> int:
        pass

    @abstractmethod
    def set_brightness(self, percentage: int) -> int:
        pass
