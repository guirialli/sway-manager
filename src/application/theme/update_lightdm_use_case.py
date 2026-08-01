from typing import Optional, List
from domain.theme.repositories import ILightDMRepository
from domain.theme.entities import LightDMSettings


class UpdateLightDMUseCase:
    def __init__(self, repository: ILightDMRepository):
        self.repository = repository

    def get_settings(self) -> LightDMSettings:
        return self.repository.get_settings()

    def execute(
        self, settings: LightDMSettings, image_source_path: Optional[str] = None
    ) -> bool:
        return self.repository.save_settings(settings, image_source_path)

    def get_available_users(self) -> List[str]:
        return self.repository.get_available_users()

    def get_available_sessions(self) -> List[str]:
        return self.repository.get_available_sessions()
