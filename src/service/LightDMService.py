import os
import shutil
import subprocess
import configparser
import shlex
from service.ThemeService import ThemeService


class LightDMService:
    CONFIG_FILE = "/etc/lightdm/lightdm-gtk-greeter.conf"
    SYSTEM_WALLPAPER = "/etc/lightdm/background.jpg"

    @classmethod
    def get_settings(cls) -> dict:
        settings = {
            "background": "/usr/share/backgrounds/lightdm-wallpaper.jpg",
            "theme-name": "Adwaita-dark",
            "icon-theme-name": "Adwaita",
            "font-name": "Sans 11",
            "cursor-theme-name": "Bibata-Modern-Ice",
            "clock-format": "%a, %d %b %H:%M",
            "draw-user-backgrounds": "false",
            "hide-user-image": "false",
        }

        if os.path.isfile(cls.CONFIG_FILE):
            try:
                config = configparser.ConfigParser(interpolation=None)
                config.read(cls.CONFIG_FILE)
                if "greeter" in config:
                    for key in settings:
                        if key in config["greeter"]:
                            settings[key] = config["greeter"][key]
            except Exception as e:
                print(f"Error reading {cls.CONFIG_FILE}: {e}")

        return settings

    @classmethod
    def save_settings(cls, new_settings: dict, image_source_path: str = None) -> bool:
        """Salva as configurações do LightDM GTK Greeter em /etc/lightdm/lightdm-gtk-greeter.conf em uma única chamada de pkexec."""
        tmp_path = "/tmp/lightdm-gtk-greeter.conf.tmp"
        try:
            cmd_parts = ["mkdir -p /etc/lightdm"]

            # 1. Copiar imagem para /etc/lightdm/background.jpg se fornecida
            if image_source_path and os.path.isfile(image_source_path):
                cmd_parts.append(
                    f"cp {shlex.quote(image_source_path)} {shlex.quote(cls.SYSTEM_WALLPAPER)}"
                )
                cmd_parts.append(f"chmod 644 {shlex.quote(cls.SYSTEM_WALLPAPER)}")
                new_settings["background"] = cls.SYSTEM_WALLPAPER

            # 2. Ler a configuração existente ou preparar nova seção [greeter]
            config = configparser.ConfigParser(interpolation=None)
            config.optionxform = str
            if os.path.isfile(cls.CONFIG_FILE):
                config.read(cls.CONFIG_FILE)

            if "greeter" not in config:
                config["greeter"] = {}

            for key, val in new_settings.items():
                config["greeter"][key] = str(val)

            # 3. Gravar arquivo temporário
            with open(tmp_path, "w") as f:
                config.write(f)

            cmd_parts.append(
                f"cp {shlex.quote(tmp_path)} {shlex.quote(cls.CONFIG_FILE)}"
            )
            cmd_parts.append(f"chmod 644 {shlex.quote(cls.CONFIG_FILE)}")

            # 4. Executar todas as alterações agrupadas em um único pkexec
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

