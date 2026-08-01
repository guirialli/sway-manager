from typing import Optional
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
