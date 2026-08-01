from presentation.gui.components.base_osd import OSD
from infrastructure.display.brightnessctl_repository import BrightnessctlRepository
from application.display.set_brightness_use_case import SetBrightnessUseCase


class BrightnessOSD(OSD):
    def __init__(self, acao: str):
        repo = BrightnessctlRepository()
        use_case = SetBrightnessUseCase(repo)
        percent = use_case.execute_change(direction=acao, step=5)
        label = "🌕"
        super().__init__(percent=percent, label=label)
