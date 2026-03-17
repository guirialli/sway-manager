from controller.BrightnessctlController import BrightnessctlController
from ui.components.OSD import OSD


class BrightnessOSD(OSD):
    def __init__(self, acao: str):
        percent = BrightnessctlController.toggle_brightness(acao)
        label = "🌕"
        super().__init__(percent=percent, label=label)
