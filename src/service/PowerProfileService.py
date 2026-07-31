import subprocess
import json


class PowerProfileService:
    PROFILES = ["power-saver", "balanced", "performance"]
    PROFILE_NAMES = {
        "power-saver": "economia de energia",
        "balanced": "equilibrado",
        "performance": "desempenho máximo",
    }
    ICONS = {"power-saver": "", "balanced": "", "performance": ""}

    @classmethod
    def get_active_profile(cls) -> str:
        try:
            res = subprocess.run(
                ["powerprofilesctl", "get"], capture_output=True, text=True
            )
            val = res.stdout.strip()
            if val in cls.PROFILES:
                return val
        except Exception:
            pass
        return "balanced"

    @classmethod
    def status(cls):
        actual = cls.get_active_profile()
        icon = cls.ICONS.get(actual, "")
        print(
            json.dumps(
                {"text": icon, "tooltip": f"Power Profile: {actual}"}
            )
        )

    @classmethod
    def toggle(cls, flag: str | None = None):
        profiles = cls.PROFILES
        current_profile = cls.get_active_profile()

        if flag == "-p":
            target = "performance"
        elif flag == "-b":
            target = "balanced"
        elif flag == "-s":
            target = "power-saver"
        else:
            idx = (
                profiles.index(current_profile)
                if current_profile in profiles
                else 0
            )
            next_idx = (idx + 1) % len(profiles)
            target = profiles[next_idx]

        subprocess.run(["powerprofilesctl", "set", target])
        msg_name = cls.PROFILE_NAMES.get(target, target)
        subprocess.run(["notify-send", f"{msg_name} setado"])
        print(f"Power profile set to: {target}")
