from typing import Optional
from domain.power.repositories import IPowerProfileRepository
from domain.power.entities import PowerProfileState


class TogglePowerProfileUseCase:
    PROFILES = ["power-saver", "balanced", "performance"]

    def __init__(self, repository: IPowerProfileRepository):
        self.repository = repository

    def get_state(self) -> PowerProfileState:
        return self.repository.get_state()

    def execute(self, flag: Optional[str] = None) -> str:
        current_profile = self.repository.get_state().active_profile

        if flag == "-p":
            target = "performance"
        elif flag == "-b":
            target = "balanced"
        elif flag == "-s":
            target = "power-saver"
        else:
            idx = (
                self.PROFILES.index(current_profile)
                if current_profile in self.PROFILES
                else 0
            )
            next_idx = (idx + 1) % len(self.PROFILES)
            target = self.PROFILES[next_idx]

        return self.repository.set_profile(target)
