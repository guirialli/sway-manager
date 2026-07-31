from ui.osd.BrightnessOSD import BrightnessOSD
from application.ApplicationFactory import ApplicationFactory
from application.BrightnessPopupApp import BrightnessPopupApp


class BrightnessOSDApp:
    def __init__(self, acao: str | None) -> None:
        if not acao or acao.lower() == "popup":
            BrightnessPopupApp()
            return

        if acao not in ("up", "down"):
            print(f"{acao} não corresponde as suportadas como: up e down.")
            return

        ApplicationFactory.buildWidget(lambda: BrightnessOSD(acao))
