import subprocess


class BrightnessctlService:
    @classmethod
    def get_brightness_cmd(cls, cmd: list[str]) -> int:
        try:
            cmd = ["brightnessctl"] + cmd
            return int(subprocess.check_output(cmd).strip())
        except Exception:
            return 0

    @classmethod
    def get_current_percentage(cls):
        cur = cls.get_brightness_cmd(["g"])
        maxv = cls.get_brightness_cmd(["m"])

        if maxv == 0:
            return 0
        return int((cur * 100) / maxv)

    @classmethod
    def set_brightness(cls, value: int):
        if value < 1:
            value = 1
        elif value > 100:
            value = 100

        subprocess.run(["brightnessctl", "set", f"{value}%"])
        return value
