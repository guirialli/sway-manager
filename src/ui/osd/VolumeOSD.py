from ui.components.OSD import OSD
from controller.MixerController import MixerController


class VolumeOSD(OSD):
    def __init__(self, acao: str):
        percernt, is_muted = MixerController.gerenciar_volume(acao)

        label = "🔇" if is_muted else "🎧"
        super().__init__(percent=int(percernt), label=label)
