import os
import subprocess
import configparser
import shlex
import getpass
from typing import Optional, List
from domain.theme.entities import LightDMSettings
from domain.theme.repositories import ILightDMRepository


class LightDMRepository(ILightDMRepository):
    GREETER_CONFIG = "/etc/lightdm/lightdm-gtk-greeter.conf"
    DAEMON_CONFIG = "/etc/lightdm/lightdm.conf"
    SYSTEM_WALLPAPER = "/etc/lightdm/background.jpg"

    GREETER_KEYS = {
        "background",
        "theme-name",
        "icon-theme-name",
        "font-name",
        "cursor-theme-name",
        "cursor-theme-size",
        "xft-antialias",
        "xft-dpi",
        "xft-hintstyle",
        "xft-rgba",
        "clock-format",
        "position",
        "active-monitor",
        "screensaver-timeout",
        "indicators",
        "draw-user-backgrounds",
        "draw-grid",
        "hide-user-image",
        "default-user-image",
    }

    DAEMON_KEYS = {
        "autologin-user",
        "autologin-user-timeout",
        "autologin-session",
        "user-session",
        "greeter-show-manual-login",
        "greeter-hide-users",
        "allow-guest",
    }

    def get_settings(self) -> LightDMSettings:
        default_dict = LightDMSettings().to_dict()

        # 1. Read Greeter Config
        if os.path.isfile(self.GREETER_CONFIG):
            try:
                config = configparser.ConfigParser(interpolation=None)
                config.read(self.GREETER_CONFIG)
                if "greeter" in config:
                    for key in self.GREETER_KEYS:
                        if key in config["greeter"]:
                            default_dict[key] = config["greeter"][key]
            except Exception as e:
                print(f"Error reading {self.GREETER_CONFIG}: {e}")

        # 2. Read Daemon Config
        if os.path.isfile(self.DAEMON_CONFIG):
            try:
                config = configparser.ConfigParser(interpolation=None)
                config.read(self.DAEMON_CONFIG)

                seat_section = None
                for sec in ["Seat:*", "SeatDefaults", "LightDM"]:
                    if sec in config:
                        seat_section = config[sec]
                        break

                if seat_section:
                    for key in self.DAEMON_KEYS:
                        if key in seat_section:
                            default_dict[key] = seat_section[key]
            except Exception as e:
                print(f"Error reading {self.DAEMON_CONFIG}: {e}")

        return LightDMSettings.from_dict(default_dict)

    def save_settings(
        self, settings: LightDMSettings, image_source_path: Optional[str] = None
    ) -> bool:
        tmp_greeter_path = "/tmp/lightdm-gtk-greeter.conf.tmp"
        tmp_daemon_path = "/tmp/lightdm.conf.tmp"
        try:
            cmd_parts = ["mkdir -p /etc/lightdm"]
            new_settings = settings.to_dict()

            if image_source_path and os.path.isfile(image_source_path):
                cmd_parts.append(
                    f"cp {shlex.quote(image_source_path)} {shlex.quote(self.SYSTEM_WALLPAPER)}"
                )
                cmd_parts.append(f"chmod 644 {shlex.quote(self.SYSTEM_WALLPAPER)}")
                new_settings["background"] = self.SYSTEM_WALLPAPER

            # 1. Write Greeter Config
            greeter_config = configparser.ConfigParser(interpolation=None)
            greeter_config.optionxform = str
            if os.path.isfile(self.GREETER_CONFIG):
                greeter_config.read(self.GREETER_CONFIG)

            if "greeter" not in greeter_config:
                greeter_config["greeter"] = {}

            for key in self.GREETER_KEYS:
                if key in new_settings:
                    val = str(new_settings[key])
                    # Avoid writing empty DPI
                    if key == "xft-dpi" and not val:
                        greeter_config["greeter"].pop(key, None)
                    else:
                        greeter_config["greeter"][key] = val

            with open(tmp_greeter_path, "w") as f:
                greeter_config.write(f)

            cmd_parts.append(
                f"cp {shlex.quote(tmp_greeter_path)} {shlex.quote(self.GREETER_CONFIG)}"
            )
            cmd_parts.append(f"chmod 644 {shlex.quote(self.GREETER_CONFIG)}")

            # 2. Write Daemon Config
            daemon_config = configparser.ConfigParser(interpolation=None)
            daemon_config.optionxform = str
            if os.path.isfile(self.DAEMON_CONFIG):
                daemon_config.read(self.DAEMON_CONFIG)

            if "Seat:*" not in daemon_config:
                daemon_config["Seat:*"] = {}

            for key in self.DAEMON_KEYS:
                if key in new_settings:
                    val = str(new_settings[key])
                    if not val and key in ("autologin-user", "autologin-session"):
                        daemon_config["Seat:*"].pop(key, None)
                    else:
                        daemon_config["Seat:*"][key] = val

            with open(tmp_daemon_path, "w") as f:
                daemon_config.write(f)

            cmd_parts.append(
                f"cp {shlex.quote(tmp_daemon_path)} {shlex.quote(self.DAEMON_CONFIG)}"
            )
            cmd_parts.append(f"chmod 644 {shlex.quote(self.DAEMON_CONFIG)}")

            full_cmd = " && ".join(cmd_parts)
            subprocess.run(
                ["pkexec", "sh", "-c", full_cmd],
                check=True,
            )

            return True
        except Exception as e:
            print(f"Error saving LightDM settings: {e}")
            return False
        finally:
            for p in [tmp_greeter_path, tmp_daemon_path]:
                if os.path.exists(p):
                    try:
                        os.remove(p)
                    except Exception:
                        pass

    def get_available_users(self) -> List[str]:
        users = []
        try:
            with open("/etc/passwd", "r") as f:
                for line in f:
                    parts = line.strip().split(":")
                    if len(parts) >= 7:
                        username = parts[0]
                        uid = int(parts[2])
                        shell = parts[6]
                        if (
                            1000 <= uid < 65534
                            and username != "nobody"
                            and shell not in (
                                "/bin/false",
                                "/usr/bin/nologin",
                                "/sbin/nologin",
                                "/bin/sync",
                            )
                        ):
                            users.append(username)
        except Exception as e:
            print(f"Error reading users from /etc/passwd: {e}")

        current_user = getpass.getuser()
        if current_user and current_user not in users and current_user != "nobody":
            users.append(current_user)

        return sorted(list(set(users)))

    def get_available_sessions(self) -> List[str]:
        sessions = set()
        session_dirs = [
            "/usr/share/wayland-sessions",
            "/usr/share/xsessions",
        ]
        for s_dir in session_dirs:
            if os.path.isdir(s_dir):
                for f_name in os.listdir(s_dir):
                    if f_name.endswith(".desktop"):
                        session_name = f_name[:-8]
                        sessions.add(session_name)

        result = sorted(list(sessions))
        # Ensure 'sway' is prioritized if available
        if "sway" in result:
            result.remove("sway")
            result.insert(0, "sway")

        return result
