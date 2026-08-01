from abc import ABC, abstractmethod
from domain.audio.value_objects import VolumeState


class IAudioRepository(ABC):
    @abstractmethod
    def get_volume_state(self) -> VolumeState:
        pass

    @abstractmethod
    def set_volume(self, percent: str) -> None:
        pass

    @abstractmethod
    def toggle_mute(self) -> None:
        pass
