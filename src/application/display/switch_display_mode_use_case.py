from domain.display.entities import MonitoresSway, DisplaySwitchType
from domain.display.repositories import IDisplayRepository


class SwitchDisplayModeUseCase:
    def __init__(self, repository: IDisplayRepository):
        self.repository = repository

    def get_monitors(self) -> MonitoresSway:
        return self.repository.get_monitors()

    def execute(self, mode: DisplaySwitchType) -> None:
        self.repository.apply_config(mode)

    def recarregar_sway(self) -> None:
        self.repository.recarregar_sway()

    def get_connected_monitors_count(self) -> int:
        return self.repository.get_connected_monitors_count()

    def get_current_layout(self) -> DisplaySwitchType:
        return self.repository.get_current_layout()
