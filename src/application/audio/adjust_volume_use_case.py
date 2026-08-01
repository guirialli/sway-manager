from domain.audio.repositories import IAudioRepository
from domain.audio.value_objects import VolumeState, AudioAction


class AdjustVolumeUseCase:
    def __init__(self, repository: IAudioRepository):
        self.repository = repository

    def get_state(self) -> VolumeState:
        return self.repository.get_volume_state()

    def execute_action(self, action: AudioAction, step_percent: int = 5) -> VolumeState:
        if action == AudioAction.UP:
            self.repository.set_volume(f"{step_percent}%+")
        elif action == AudioAction.DOWN:
            self.repository.set_volume(f"{step_percent}%-")
        elif action == AudioAction.MUTE:
            self.repository.toggle_mute()

        return self.repository.get_volume_state()
