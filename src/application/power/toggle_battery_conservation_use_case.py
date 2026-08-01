from domain.power.repositories import IBatteryRepository
from domain.power.entities import BatteryState


class ToggleBatteryConservationUseCase:
    def __init__(self, repository: IBatteryRepository):
        self.repository = repository

    def get_state(self) -> BatteryState:
        return self.repository.get_state()

    def execute(self) -> tuple[bool, str]:
        return self.repository.toggle()
