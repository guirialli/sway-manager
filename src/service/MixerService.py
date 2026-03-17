import subprocess


class MixerService:
    @classmethod
    def _get_status(cls):
        resultado = subprocess.run(
            ["wpctl", "get-volume", "@DEFAULT_AUDIO_SINK@"],
            text=True,
            capture_output=True,
        )

        return resultado.stdout.strip()

    @classmethod
    def get_is_muted(cls) -> bool:
        saida = cls._get_status()
        return "[MUTED]" in saida

    @classmethod
    def get_volume(cls) -> float:
        saida = cls._get_status()
        volume = 0.0
        for palava in saida.split():
            try:
                volume = float(palava) * 100
                break
            except Exception:
                continue

        return volume

    @classmethod
    # percent deve seguir o padrão do wpctl usando 5%+ por exemplo para aumentar 5 porcento
    # Ou 5%- para reduzir 5 porcento
    def set_volume(cls, percent: str):
        subprocess.run(["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", percent])

    @classmethod
    def toggle_volume_muted(cls):
        subprocess.run(["wpctl", "set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
