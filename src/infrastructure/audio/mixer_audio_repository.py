import subprocess
from domain.audio.value_objects import VolumeState
from domain.audio.repositories import IAudioRepository


class MixerAudioRepository(IAudioRepository):
    def _get_status(self) -> str:
        resultado = subprocess.run(
            ["wpctl", "get-volume", "@DEFAULT_AUDIO_SINK@"],
            text=True,
            capture_output=True,
        )
        return resultado.stdout.strip()

    def get_volume_state(self) -> VolumeState:
        saida = self._get_status()
        is_muted = "[MUTED]" in saida
        volume = 0.0
        for palavra in saida.split():
            try:
                volume = float(palavra) * 100
                break
            except Exception:
                continue

        return VolumeState(percentage=volume, is_muted=is_muted)

    def set_volume(self, percent: str) -> None:
        subprocess.run(["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", percent])

    def toggle_mute(self) -> None:
        subprocess.run(["wpctl", "set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
