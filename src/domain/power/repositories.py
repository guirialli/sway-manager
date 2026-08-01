from abc import ABC, abstractmethod
from typing import Optional
from domain.power.entities import BatteryState, IdleState, PowerProfileState


class IBatteryRepository(ABC):
    @abstractmethod
    def get_state(self) -> BatteryState:
        pass

    @abstractmethod
    def toggle(self) -> tuple[bool, str]:
        pass


class IIdleRepository(ABC):
    @abstractmethod
    def get_state(self) -> IdleState:
        pass

    @abstractmethod
    def toggle(self, flag: Optional[str] = None) -> str:
        pass


class IPowerProfileRepository(ABC):
    @abstractmethod
    def get_state(self) -> PowerProfileState:
        pass

    @abstractmethod
    def set_profile(self, target: str) -> str:
        pass
