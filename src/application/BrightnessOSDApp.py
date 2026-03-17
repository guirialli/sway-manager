from ui.osd.BrightnessOSD import BrightnessOSD
from application.ApplicationFactory import ApplicationFactory


class BrightnessOSDApp:
    def __init__(self, acao: str | None) -> None:
        if not acao:
            print("Ação não informada!")
            return

        if acao not in ("up", "down"):
            print(f"{acao} não corresponde as suportada como: up e down.")
            return

        ApplicationFactory.buildWidget(lambda: BrightnessOSD(acao))
