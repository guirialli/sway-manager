import os
import datetime
import subprocess
from domain.media.value_objects import ScreenshotMode
from domain.media.repositories import IScreenshotRepository


class GrimSlurpScreenshotRepository(IScreenshotRepository):
    def take_screenshot(self, mode: ScreenshotMode) -> None:
        try:
            res = subprocess.run(
                ["xdg-user-dir", "PICTURES"], capture_output=True, text=True
            )
            pictures_dir = res.stdout.strip()
            if not pictures_dir:
                pictures_dir = os.path.expanduser("~/Pictures")
        except Exception:
            pictures_dir = os.path.expanduser("~/Pictures")

        folder = os.path.join(pictures_dir, "screenshots")
        os.makedirs(folder, exist_ok=True)

        timestamp = datetime.datetime.now().strftime("%Y-%m-%d_%H:%M:%S")
        filename = os.path.join(folder, f"screenshot_{timestamp}.png")

        if mode == ScreenshotMode.AREA:
            cmd = f'grim -g "$(slurp)" "{filename}"'
            action_desc = "da área selecionada"
        elif mode == ScreenshotMode.WINDOW:
            cmd = f'grim -g "$(swaymsg -t get_tree | jq -r \'.. | select(.focused? == true).rect | "\\(.x),\\(.y) \\(.width)x\\(.height)"\')" "{filename}"'
            action_desc = "da janela"
        else:
            cmd = f'grim -g "$(slurp -o)" "{filename}"'
            action_desc = "da tela"

        res = subprocess.run(cmd, shell=True)

        if res.returncode == 0 and os.path.isfile(filename):
            subprocess.run(f"wl-copy < '{filename}'", shell=True)
            subprocess.run(
                ["notify-send", f"Captura {action_desc} salva em {filename}"]
            )
            print(f"Screenshot saved to {filename}")
        else:
            print("Screenshot cancelled or failed.")
