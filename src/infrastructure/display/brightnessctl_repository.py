import subprocess
from domain.display.repositories import IBrightnessRepository


class BrightnessctlRepository(IBrightnessRepository):
    def _get_brightness_cmd(self, cmd: list[str]) -> int:
        try:
            full_cmd = ["brightnessctl"] + cmd
            return int(subprocess.check_output(full_cmd).strip())
        except Exception:
            return 0

    def get_current_percentage(self) -> int:
        cur = self._get_brightness_cmd(["g"])
        maxv = self._get_brightness_cmd(["m"])

        if maxv == 0:
            return 0
        return int((cur * 100) / maxv)

    def set_brightness(self, percentage: int) -> int:
        if percentage < 1:
            percentage = 1
        elif percentage > 100:
            percentage = 100

        subprocess.run(["brightnessctl", "set", f"{percentage}%"])
        return percentage
