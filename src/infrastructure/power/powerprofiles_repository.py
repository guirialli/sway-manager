import subprocess
from domain.power.entities import PowerProfileState
from domain.power.repositories import IPowerProfileRepository


class PowerProfilesRepository(IPowerProfileRepository):
    PROFILES = ["power-saver", "balanced", "performance"]

    def get_state(self) -> PowerProfileState:
        try:
            res = subprocess.run(
                ["powerprofilesctl", "get"], capture_output=True, text=True
            )
            val = res.stdout.strip()
            if val in self.PROFILES:
                return PowerProfileState(active_profile=val)
        except Exception:
            pass
        return PowerProfileState(active_profile="balanced")

    def set_profile(self, target: str) -> str:
        subprocess.run(["powerprofilesctl", "set", target])
        msg_name = PowerProfileState.PROFILE_NAMES.get(target, target)
        subprocess.run(["notify-send", f"{msg_name} setado"])
        return f"Power profile set to: {target}"
