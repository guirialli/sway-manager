from ui.osd.VolumeOSD import VolumeOSD
from application.ApplicationFactory import ApplicationFactory


class VolumeOSDApp:
    def __init__(self, acao: str | None) -> None:
        if not acao:
            print("Ação não foi informada!")
            return

        if acao not in ("mute", "up", "down"):
            print(f"Ação não é valida {acao}, use: mute, up, donw")
            return

        ApplicationFactory.buildWidget(lambda: VolumeOSD(acao))
