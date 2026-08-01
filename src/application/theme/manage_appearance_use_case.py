from domain.theme.repositories import IThemeRepository
from domain.theme.entities import AppearanceSettings, AvailableAppearanceOptions


class ManageAppearanceUseCase:
    def __init__(self, repository: IThemeRepository):
        self.repository = repository

    def get_settings(self) -> AppearanceSettings:
        return self.repository.get_appearance_settings()

    def get_available_options(self) -> AvailableAppearanceOptions:
        return self.repository.get_available_options()

    def apply(self, settings: AppearanceSettings) -> bool:
        return self.repository.apply_appearance(settings)
