import os
import subprocess
import configparser
import shlex
from typing import Optional
from domain.theme.entities import LightDMSettings
from domain.theme.repositories import ILightDMRepository


class LightDMRepository(ILightDMRepository):
    CONFIG_FILE = "/etc/lightdm/lightdm-gtk-greeter.conf"
    SYSTEM_WALLPAPER = "/etc/lightdm/background.jpg"

    def get_settings(self) -> LightDMSettings:
        default_dict = LightDMSettings().to_dict()

        if os.path.isfile(self.CONFIG_FILE):
            try:
                config = configparser.ConfigParser(interpolation=None)
                config.read(self.CONFIG_FILE)
                if "greeter" in config:
                    for key in default_dict:
                        if key in config["greeter"]:
                            default_dict[key] = config["greeter"][key]
            except Exception as e:
                print(f"Error reading {self.CONFIG_FILE}: {e}")

        return LightDMSettings.from_dict(default_dict)

    def save_settings(
        self, settings: LightDMSettings, image_source_path: Optional[str] = None
    ) -> bool:
        tmp_path = "/tmp/lightdm-gtk-greeter.conf.tmp"
        try:
            cmd_parts = ["mkdir -p /etc/lightdm"]
            new_settings = settings.to_dict()

            if image_source_path and os.path.isfile(image_source_path):
                cmd_parts.append(
                    f"cp {shlex.quote(image_source_path)} {shlex.quote(self.SYSTEM_WALLPAPER)}"
                )
                cmd_parts.append(f"chmod 644 {shlex.quote(self.SYSTEM_WALLPAPER)}")
                new_settings["background"] = self.SYSTEM_WALLPAPER

            config = configparser.ConfigParser(interpolation=None)
            config.optionxform = str
            if os.path.isfile(self.CONFIG_FILE):
                config.read(self.CONFIG_FILE)

            if "greeter" not in config:
                config["greeter"] = {}

            for key, val in new_settings.items():
                config["greeter"][key] = str(val)

            with open(tmp_path, "w") as f:
                config.write(f)

            cmd_parts.append(
                f"cp {shlex.quote(tmp_path)} {shlex.quote(self.CONFIG_FILE)}"
            )
            cmd_parts.append(f"chmod 644 {shlex.quote(self.CONFIG_FILE)}")

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
            if os.path.exists(tmp_path):
                os.remove(tmp_path)
