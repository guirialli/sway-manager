from domain.theme.repositories import IThemeRepository
from domain.theme.entities import ThemeState


class ToggleThemeUseCase:
    def __init__(self, repository: IThemeRepository):
        self.repository = repository

    def get_state(self) -> ThemeState:
        return self.repository.get_state()

    def execute(self) -> str:
        return self.repository.toggle()
