import os
import subprocess
from typing import Optional
from domain.power.entities import IdleState
from domain.power.repositories import IIdleRepository


class SwayIdleRepository(IIdleRepository):
    STATE_FILE = os.path.expanduser("~/.cache/swayidle_enabled")
    TIMEOUT_LOCK = 180  # 3 minutes
    TIMEOUT_SLEEP = 300  # 5 minutes

    def _is_running(self) -> bool:
        res = subprocess.run(
            ["pgrep", "-x", "swayidle"], capture_output=True, text=True
        )
        return res.returncode == 0

    def get_state(self) -> IdleState:
        return IdleState(is_running=self._is_running())

    def toggle(self, flag: Optional[str] = None) -> str:
        if flag == "-n":
            subprocess.run(["systemctl", "suspend"])
            state = "off"
        elif flag == "-s":
            state = "off"
        elif flag == "-r":
            state = "off" if self._is_running() else "on"
        else:
            state = "off" if self._is_running() else "on"

        # Kill existing swayidle processes
        subprocess.run(["killall", "-q", "swayidle"])

        os.makedirs(os.path.dirname(self.STATE_FILE), exist_ok=True)

        if state == "off":
            with open(self.STATE_FILE, "w") as f:
                f.write("off")
            msg = "Suspensão desativada!"
        else:
            cmd = [
                "swayidle",
                "-w",
                "timeout",
                str(self.TIMEOUT_LOCK),
                "swaylock -f",
                "timeout",
                str(self.TIMEOUT_SLEEP),
                'swaymsg "output * dpms off"',
                "resume",
                'swaymsg "output * dpms on"',
                "before-sleep",
                "swaylock -f",
            ]
            subprocess.Popen(cmd)
            with open(self.STATE_FILE, "w") as f:
                f.write("on")
            msg = "Suspensão ativada!"

        if flag != "-s":
            subprocess.run(["notify-send", msg])
        return msg
