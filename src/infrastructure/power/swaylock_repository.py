import os
import shutil
import subprocess
from typing import Optional
from domain.power.repositories import ILockRepository
from domain.theme.repositories import IWallpaperRepository, IThemeRepository
from infrastructure.theme.sway_wallpaper_repository import SwayWallpaperRepository
from infrastructure.theme.gtk_qt_theme_repository import GtkQtThemeRepository


class SwayLockRepository(ILockRepository):
    def __init__(
        self,
        wallpaper_repo: Optional[IWallpaperRepository] = None,
        theme_repo: Optional[IThemeRepository] = None,
    ):
        self.wallpaper_repo = wallpaper_repo or SwayWallpaperRepository()
        self.theme_repo = theme_repo or GtkQtThemeRepository()

    def _is_swaylock_running(self) -> bool:
        res = subprocess.run(
            ["pgrep", "-x", "swaylock"], capture_output=True, text=True
        )
        return res.returncode == 0

    def lock_screen(self) -> None:
        if self._is_swaylock_running():
            return

        swaylock_bin = shutil.which("swaylock") or "swaylock"

        cmd = [swaylock_bin]

        # 1. Wallpaper customization
        wallpaper_path = self.wallpaper_repo.get_current_wallpaper()
        if wallpaper_path and os.path.isfile(wallpaper_path):
            cmd.extend(["-i", wallpaper_path, "-s", "fill"])
        else:
            symlink_wp = os.path.expanduser("~/.config/sway/wallpaper")
            if os.path.exists(symlink_wp):
                cmd.extend(["-i", symlink_wp, "-s", "fill"])

        # 2. System theme customization (Dark vs Light)
        current_theme = self.theme_repo.get_state().current_theme
        if current_theme == "dark":
            cmd.extend([
                "-c", "1a0a28",
                "--ring-color", "6c238eff",
                "--inside-color", "1a0a28aa",
                "--line-color", "00000000",
                "--text-color", "c3e88dff",
                "--key-hl-color", "00bcd4ff",
                "--bs-hl-color", "ff3333ff",
                "--ring-ver-color", "6c238eff",
                "--inside-ver-color", "1a0a28aa",
                "--ring-wrong-color", "ff3333ff",
                "--inside-wrong-color", "1a0a28aa",
                "--text-wrong-color", "ff3333ff",
                "--ring-clear-color", "00bcd4ff",
                "--inside-clear-color", "1a0a28aa",
            ])
        else:
            cmd.extend([
                "-c", "f5f5f7",
                "--ring-color", "1e66f5ff",
                "--inside-color", "ffffffaa",
                "--line-color", "00000000",
                "--text-color", "4c4f69ff",
                "--key-hl-color", "40a02bff",
                "--bs-hl-color", "d20f39ff",
                "--ring-ver-color", "1e66f5ff",
                "--inside-ver-color", "ffffffaa",
                "--ring-wrong-color", "d20f39ff",
                "--inside-wrong-color", "ffffffaa",
                "--text-wrong-color", "d20f39ff",
                "--ring-clear-color", "40a02bff",
                "--inside-clear-color", "ffffffaa",
            ])

        # Indicator & font settings
        cmd.extend([
            "--indicator-idle-visible",
            "--indicator-caps-lock",
            "-F",
            "--font", "MesloLGS NF"
        ])

        # 3. Dedicated swayidle process for turning off monitors after 60s of inactivity on lock screen
        idle_proc = None
        if shutil.which("swayidle"):
            try:
                idle_cmd = [
                    "swayidle",
                    "-w",
                    "timeout", "60", 'swaymsg "output * dpms off"',
                    "resume", 'swaymsg "output * dpms on"'
                ]
                idle_proc = subprocess.Popen(idle_cmd)
            except Exception as e:
                print(f"Erro ao iniciar swayidle para lockscreen: {e}")

        try:
            subprocess.run(cmd)
        finally:
            if idle_proc:
                try:
                    idle_proc.terminate()
                    idle_proc.wait(timeout=1)
                except Exception:
                    idle_proc.kill()
            subprocess.run(["swaymsg", "output * dpms on"], capture_output=True)
