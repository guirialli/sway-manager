from service.MixerService import MixerService


class MixerController:
    @classmethod
    def gerenciar_volume(cls, acao: str) -> list[float | bool]:
        if acao != "mute" and MixerService.get_is_muted():
            MixerService.toggle_volume_muted()

        volume = MixerService.get_volume()
        if acao == "up":
            volume += 5
            if volume >= 100:
                volume = 100
        elif acao == "down":
            volume -= 5
            if volume < 0:
                volume = 0
        elif acao == "mute":
            MixerService.toggle_volume_muted()

        MixerService.set_volume(f"{volume}%")
        volume = MixerService.get_volume()
        is_muted = MixerService.get_is_muted()

        return [volume, is_muted]
