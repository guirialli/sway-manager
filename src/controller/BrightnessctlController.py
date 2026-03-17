from service.BrightnessctlService import BrightnessctlService


class BrightnessctlController:
    @classmethod
    def toggle_brightness(cls, acao: str):
        curr = BrightnessctlService.get_current_percentage()

        if acao == "up":
            curr += 5
        elif acao == "down":
            curr -= 5

        if curr > 100:
            curr = 100
        elif curr < 0:
            curr = 0

        return BrightnessctlService.set_brightness(curr)
