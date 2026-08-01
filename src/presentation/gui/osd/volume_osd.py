from presentation.gui.components.base_osd import OSD
from infrastructure.audio.mixer_audio_repository import MixerAudioRepository
from application.audio.adjust_volume_use_case import AdjustVolumeUseCase
from domain.audio.value_objects import AudioAction


class VolumeOSD(OSD):
    def __init__(self, acao: str):
        repo = MixerAudioRepository()
        use_case = AdjustVolumeUseCase(repo)
        
        if acao == "up":
            state = use_case.execute_action(AudioAction.UP)
        elif acao == "down":
            state = use_case.execute_action(AudioAction.DOWN)
        elif acao == "mute":
            state = use_case.execute_action(AudioAction.MUTE)
        else:
            state = use_case.get_state()

        label = "🔇" if state.is_muted else "🎧"
        super().__init__(percent=int(state.percentage), label=label)
