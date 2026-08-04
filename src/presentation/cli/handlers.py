import json
import os
from utils.array import ArrayUtils

# Infrastructure
from infrastructure.display.brightnessctl_repository import BrightnessctlRepository
from infrastructure.display.sway_display_repository import SwayDisplayRepository
from infrastructure.audio.mixer_audio_repository import MixerAudioRepository
from infrastructure.power.sysfs_battery_repository import SysfsBatteryRepository
from infrastructure.power.swayidle_repository import SwayIdleRepository
from infrastructure.power.swaylock_repository import SwayLockRepository
from infrastructure.power.powerprofiles_repository import PowerProfilesRepository
from infrastructure.theme.gtk_qt_theme_repository import GtkQtThemeRepository
from infrastructure.theme.sway_wallpaper_repository import SwayWallpaperRepository
from infrastructure.media.grim_slurp_screenshot_repository import GrimSlurpScreenshotRepository
from infrastructure.menu.wofi_launcher import WofiRepository
from infrastructure.clipboard.cliphist_repository import CliphistRepository

# Application Use Cases
from application.display.set_brightness_use_case import SetBrightnessUseCase
from application.display.switch_display_mode_use_case import SwitchDisplayModeUseCase
from application.audio.adjust_volume_use_case import AdjustVolumeUseCase
from application.power.toggle_battery_conservation_use_case import ToggleBatteryConservationUseCase
from application.power.toggle_idle_use_case import ToggleIdleUseCase
from application.power.lock_screen_use_case import LockScreenUseCase
from application.power.toggle_power_profile_use_case import TogglePowerProfileUseCase
from application.theme.toggle_theme_use_case import ToggleThemeUseCase
from application.theme.set_wallpaper_use_case import SetWallpaperUseCase
from application.media.take_screenshot_use_case import TakeScreenshotUseCase
from application.menu.show_menu_use_case import ShowMenuUseCase
from application.clipboard.manage_clipboard_use_case import ManageClipboardUseCase
from domain.media.value_objects import ScreenshotMode

# Presentation GUI
from presentation.gui.app_factory import ApplicationFactory


class CLIHandlers:
    @staticmethod
    def handle_settings():
        from presentation.gui.windows.config_center import ConfigCenterWindow
        ApplicationFactory.buildWidget(lambda: ConfigCenterWindow())

    @staticmethod
    def handle_monitor(args: list[str]):
        from presentation.gui.windows.monitor_swap import MonitorSwapWindow
        ApplicationFactory.buildWidget(
            lambda: MonitorSwapWindow(), desktop_file_name="sway.apps.monitor-swap"
        )

    @staticmethod
    def handle_wallpaper(args: list[str]):
        pasta = ArrayUtils.getSafe(args, 2)
        if not pasta:
            use_case = SetWallpaperUseCase(SwayWallpaperRepository())
            pasta = use_case.get_wallpaper_folder()
        from presentation.gui.windows.wallpaper_picker import WallpaperPickerWindow
        ApplicationFactory.buildWidget(lambda: WallpaperPickerWindow(pasta_imagem=pasta))

    @staticmethod
    def handle_osd(args: list[str]):
        osd_type = ArrayUtils.getSafe(args, 2)
        action = ArrayUtils.getSafe(args, 3)
        if not osd_type:
            print("OSD não informado!")
            return

        if osd_type == "brilho":
            if not action or action.lower() == "popup":
                from presentation.gui.windows.brightness_popup import BrightnessPopup
                ApplicationFactory.buildWidget(lambda: BrightnessPopup())
            else:
                from presentation.gui.osd.brightness_osd import BrightnessOSD
                ApplicationFactory.buildWidget(lambda: BrightnessOSD(action))
        elif osd_type == "volume":
            from presentation.gui.osd.volume_osd import VolumeOSD
            ApplicationFactory.buildWidget(lambda: VolumeOSD(action))

    @staticmethod
    def handle_brightness(args: list[str]):
        action = ArrayUtils.getSafe(args, 2)
        if not action or action.lower() == "popup":
            from presentation.gui.windows.brightness_popup import BrightnessPopup
            ApplicationFactory.buildWidget(lambda: BrightnessPopup())
        else:
            from presentation.gui.osd.brightness_osd import BrightnessOSD
            ApplicationFactory.buildWidget(lambda: BrightnessOSD(action))


    @staticmethod
    def handle_battery(args: list[str]):
        action = ArrayUtils.getSafe(args, 2, "toggle").lower()
        use_case = ToggleBatteryConservationUseCase(SysfsBatteryRepository())
        if action == "status":
            state = use_case.get_state()
            print(json.dumps(state.to_waybar_json()))
        else:
            ok, msg = use_case.execute()
            print(msg)

    @staticmethod
    def handle_idle(args: list[str]):
        action = ArrayUtils.getSafe(args, 2, "toggle").lower()
        flag = ArrayUtils.getSafe(args, 3)
        use_case = ToggleIdleUseCase(SwayIdleRepository())
        if action == "status":
            state = use_case.get_state()
            print(json.dumps(state.to_waybar_json()))
        else:
            msg = use_case.execute(flag)
            print(msg)

    @staticmethod
    def handle_theme(args: list[str]):
        action = ArrayUtils.getSafe(args, 2, "toggle").lower()
        use_case = ToggleThemeUseCase(GtkQtThemeRepository())
        if action == "status":
            state = use_case.get_state()
            print(json.dumps(state.to_waybar_json()))
        else:
            msg = use_case.execute()
            print(msg)

    @staticmethod
    def handle_power(args: list[str]):
        action = ArrayUtils.getSafe(args, 2, "toggle").lower()
        flag = ArrayUtils.getSafe(args, 3)
        use_case = TogglePowerProfileUseCase(PowerProfilesRepository())
        if action == "status":
            state = use_case.get_state()
            print(json.dumps(state.to_waybar_json()))
        else:
            msg = use_case.execute(flag)
            print(msg)

    @staticmethod
    def handle_screenshot(args: list[str]):
        raw_mode = ArrayUtils.getSafe(args, 2, "full").lower()
        if raw_mode in ("area", "-g"):
            mode = ScreenshotMode.AREA
        elif raw_mode in ("window", "-w"):
            mode = ScreenshotMode.WINDOW
        else:
            mode = ScreenshotMode.FULL

        use_case = TakeScreenshotUseCase(GrimSlurpScreenshotRepository())
        use_case.execute(mode)

    @staticmethod
    def handle_menu(args: list[str]):
        category_filter = ArrayUtils.getSafe(args, 2)
        use_case = ShowMenuUseCase(WofiRepository())
        use_case.execute(category_filter=category_filter)

    @staticmethod
    def handle_clipboard(args: list[str]):
        action = ArrayUtils.getSafe(args, 2)
        use_case = ManageClipboardUseCase(CliphistRepository())
        use_case.execute(action=action)

    @staticmethod
    def handle_lock(args: list[str]):
        use_case = LockScreenUseCase(SwayLockRepository())
        use_case.execute()


