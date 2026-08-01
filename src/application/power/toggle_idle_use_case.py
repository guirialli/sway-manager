from typing import Optional
from domain.power.repositories import IIdleRepository
from domain.power.entities import IdleState


class ToggleIdleUseCase:
    def __init__(self, repository: IIdleRepository):
        self.repository = repository

    def get_state(self) -> IdleState:
        return self.repository.get_state()

    def execute(self, flag: Optional[str] = None) -> str:
        return self.repository.toggle(flag)
