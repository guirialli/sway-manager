import os
import subprocess
import json


class IdleService:
    STATE_FILE = os.path.expanduser("~/.cache/swayidle_enabled")
    TIMEOUT_LOCK = 180  # 3 minutes
    TIMEOUT_SLEEP = 300  # 5 minutes

    @classmethod
    def _is_running(cls) -> bool:
        res = subprocess.run(
            ["pgrep", "-x", "swayidle"], capture_output=True, text=True
        )
        return res.returncode == 0

    @classmethod
    def status(cls):
        if not cls._is_running():
            print(
                json.dumps(
                    {
                        "text": "",
                        "tooltip": "Suspensão desativada",
                        "class": "idle-off",
                    }
                )
            )
        else:
            print(
                json.dumps(
                    {
                        "text": "",
                        "tooltip": "Suspensão ativada",
                        "class": "idle-on",
                    }
                )
            )

    @classmethod
    def toggle(cls, flag: str | None = None):
        if flag == "-n":
            subprocess.run(["systemctl", "suspend"])
            state = "off"
        elif flag == "-s":
            state = "off"
        elif flag == "-r":
            state = "off" if cls._is_running() else "on"
        else:
            state = "off" if cls._is_running() else "on"

        # Kill existing swayidle processes
        subprocess.run(["killall", "-q", "swayidle"])

        os.makedirs(os.path.dirname(cls.STATE_FILE), exist_ok=True)

        if state == "off":
            with open(cls.STATE_FILE, "w") as f:
                f.write("off")
            msg = "Suspensão desativada!"
        else:
            cmd = [
                "swayidle",
                "-w",
                "timeout",
                str(cls.TIMEOUT_LOCK),
                "swaylock -f",
                "timeout",
                str(cls.TIMEOUT_SLEEP),
                'swaymsg "output * dpms off"',
                "resume",
                'swaymsg "output * dpms on"',
                "before-sleep",
                "swaylock -f",
            ]
            subprocess.Popen(cmd)
            with open(cls.STATE_FILE, "w") as f:
                f.write("on")
            msg = "Suspensão ativada!"

        if flag != "-s":
            subprocess.run(["notify-send", msg])
        print(msg)
