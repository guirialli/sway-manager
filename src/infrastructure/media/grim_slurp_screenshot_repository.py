import os
import uuid
import json
import datetime
import subprocess
from typing import Optional
from PySide6.QtGui import QImage
from PySide6.QtCore import QRect

from domain.media.value_objects import ScreenshotMode
from domain.media.repositories import IScreenshotRepository
from domain.notification.entities import Notification
from domain.notification.repositories import INotificationRepository
from infrastructure.notification.desktop_notification_repository import (
    DesktopNotificationRepository,
)


class GrimSlurpScreenshotRepository(IScreenshotRepository):
    def __init__(self, notification_repo: Optional[INotificationRepository] = None):
        self.notification_repo = notification_repo or DesktopNotificationRepository()

    def _parse_geometry(self, geom_str: str) -> Optional[tuple[int, int, int, int]]:
        try:
            parts = geom_str.strip().split()
            if len(parts) != 2:
                return None
            x_str, y_str = parts[0].split(",")
            w_str, h_str = parts[1].split("x")
            return int(x_str), int(y_str), int(w_str), int(h_str)
        except Exception:
            return None

    def _find_focused_rect(self, node: dict) -> Optional[tuple[int, int, int, int]]:
        if node.get("focused"):
            rect = node.get("rect")
            if (
                rect
                and "x" in rect
                and "y" in rect
                and "width" in rect
                and "height" in rect
            ):
                return (
                    int(rect["x"]),
                    int(rect["y"]),
                    int(rect["width"]),
                    int(rect["height"]),
                )
        for child in node.get("nodes", []) + node.get("floating_nodes", []):
            res = self._find_focused_rect(child)
            if res:
                return res
        return None

    def _get_focused_window_rect(self) -> Optional[tuple[int, int, int, int]]:
        try:
            res = subprocess.run(
                ["swaymsg", "-t", "get_tree"], capture_output=True, text=True
            )
            if res.returncode == 0 and res.stdout:
                tree = json.loads(res.stdout)
                return self._find_focused_rect(tree)
        except Exception:
            pass
        return None

    def _crop_and_save(
        self, src_path: str, dst_path: str, rect: tuple[int, int, int, int]
    ) -> bool:
        img = QImage(src_path)
        if img.isNull():
            return False
        x, y, w, h = rect
        cropped = img.copy(QRect(x, y, w, h))
        return cropped.save(dst_path)

    def _take_slurp_selection(self, dst_path: str, frozen_path: str) -> bool:
        """Fallback to the compositor-native selector when Qt is unavailable."""
        try:
            slurp_res = subprocess.run(["slurp"], capture_output=True, text=True)
        except OSError:
            return False
        if slurp_res.returncode != 0 or not slurp_res.stdout.strip():
            return False

        geometry = self._parse_geometry(slurp_res.stdout)
        if geometry is None:
            return False

        x, y, width, height = geometry
        try:
            grim_res = subprocess.run(
                ["grim", "-g", f"{x},{y} {width}x{height}", dst_path],
                capture_output=True,
            )
        except OSError:
            return self._crop_and_save(frozen_path, dst_path, geometry)
        if grim_res.returncode == 0 and os.path.isfile(dst_path):
            return True
        return self._crop_and_save(frozen_path, dst_path, geometry)

    def get_screenshot_folder(self) -> str:
        from infrastructure.config.json_config_repository import JsonConfigRepository

        config_repo = JsonConfigRepository()
        saved_folder = config_repo.get_setting("screenshot_folder")
        if saved_folder:
            expanded = os.path.expanduser(saved_folder)
            if os.path.isdir(expanded):
                return expanded

        try:
            res = subprocess.run(
                ["xdg-user-dir", "PICTURES"], capture_output=True, text=True
            )
            pictures_dir = res.stdout.strip()
            if not pictures_dir:
                pictures_dir = os.path.expanduser("~/Pictures")
        except Exception:
            pictures_dir = os.path.expanduser("~/Pictures")

        return os.path.join(pictures_dir, "screenshots")

    def set_screenshot_folder(self, folder_path: str) -> None:
        from infrastructure.config.json_config_repository import JsonConfigRepository

        expanded = os.path.expanduser(folder_path)
        config_repo = JsonConfigRepository()
        config_repo.set_setting("screenshot_folder", expanded)

    def take_screenshot(self, mode: ScreenshotMode) -> None:
        folder = self.get_screenshot_folder()
        os.makedirs(folder, exist_ok=True)

        timestamp = datetime.datetime.now().strftime("%Y-%m-%d_%H:%M:%S")
        filename = os.path.join(folder, f"screenshot_{timestamp}.png")
        full_freeze_file = f"/tmp/sway_manager_freeze_{uuid.uuid4().hex[:8]}.png"

        freeze_files: dict[str, str] = {}
        temp_files: list[str] = [full_freeze_file]

        try:
            # Capture full desktop screenshot as fallback
            subprocess.run(["grim", full_freeze_file], capture_output=True)

            # Query outputs to capture 1:1 physical pixel snapshot per monitor
            try:
                out_res = subprocess.run(
                    ["swaymsg", "-t", "get_outputs"], capture_output=True, text=True
                )
                if out_res.returncode == 0 and out_res.stdout:
                    outputs = json.loads(out_res.stdout)
                    for o in outputs:
                        o_name = o.get("name")
                        if o_name and o.get("active", True):
                            o_file = f"/tmp/sway_manager_freeze_{o_name}_{uuid.uuid4().hex[:8]}.png"
                            grim_res = subprocess.run(
                                ["grim", "-o", o_name, o_file], capture_output=True
                            )
                            if grim_res.returncode == 0 and os.path.isfile(o_file):
                                freeze_files[o_name] = o_file
                                temp_files.append(o_file)
            except Exception:
                pass

            action_desc = "da tela"
            saved_ok = False

            if mode == ScreenshotMode.AREA:
                action_desc = "da área selecionada"
                try:
                    from presentation.gui.widgets.freeze_selection_overlay import (
                        FreezeSelectionOverlay,
                    )

                    saved_ok = FreezeSelectionOverlay.select_area(freeze_files, filename)
                except (ImportError, OSError, RuntimeError):
                    saved_ok = self._take_slurp_selection(filename, full_freeze_file)

                if not saved_ok:
                    print("Screenshot cancelled or failed.")
                    return

            elif mode == ScreenshotMode.WINDOW:
                action_desc = "da janela"
                rect = self._get_focused_window_rect()
                if rect:
                    saved_ok = self._crop_and_save(full_freeze_file, filename, rect)
                else:
                    img = QImage(full_freeze_file)
                    saved_ok = img.save(filename)

            else:  # ScreenshotMode.FULL
                action_desc = "da tela"
                img = QImage(full_freeze_file)
                saved_ok = img.save(filename)

            if saved_ok and os.path.isfile(filename):
                subprocess.run(f"wl-copy --type image/png < '{filename}'", shell=True)
                try:
                    from PySide6.QtGui import QGuiApplication, QImage
                    if QGuiApplication.instance():
                        cb = QGuiApplication.clipboard()
                        if cb:
                            cb.setImage(QImage(filename))
                except Exception:
                    pass
                self.notification_repo.notify(
                    Notification(title=f"Captura {action_desc} salva em {filename}")
                )
                print(f"Screenshot saved to {filename}")
            else:
                print("Screenshot cancelled or failed.")
        finally:
            for f in temp_files:
                if os.path.isfile(f):
                    try:
                        os.remove(f)
                    except Exception:
                        pass
