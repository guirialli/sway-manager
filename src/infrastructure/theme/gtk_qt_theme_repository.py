import os
import shutil
import subprocess
import configparser
from domain.theme.entities import ThemeState
from domain.theme.repositories import IThemeRepository


class GtkQtThemeRepository(IThemeRepository):
    WORK_PATH = os.path.expanduser("~/.config/theme_toggle")
    FILE = os.path.join(WORK_PATH, "theme")
    FOOT_DIR = os.path.expanduser("~/.config/foot")
    FOOT_FILE = os.path.join(FOOT_DIR, "foot.ini")

    def get_state(self) -> ThemeState:
        if os.path.isfile(self.FILE):
            try:
                with open(self.FILE, "r") as f:
                    return ThemeState(current_theme=f.read().strip())
            except Exception:
                pass
        return ThemeState(current_theme="light")

    def toggle(self) -> str:
        current = self.get_state().current_theme
        new_theme = "dark" if current == "light" else "light"

        gtk4 = "prefer-dark" if new_theme == "dark" else "prefer-light"
        gtk3 = "Adwaita-dark" if new_theme == "dark" else "Adwaita"
        foot_theme = "black.ini" if new_theme == "dark" else "white.ini"

        # Apply GTK theme
        subprocess.run(
            [
                "gsettings",
                "set",
                "org.gnome.desktop.interface",
                "color-scheme",
                gtk4,
            ]
        )
        subprocess.run(
            ["gsettings", "set", "org.gnome.desktop.interface", "gtk-theme", gtk3]
        )

        # Apply Foot terminal theme
        theme_src = os.path.join(self.FOOT_DIR, "themes", foot_theme)
        if os.path.isfile(theme_src):
            try:
                if os.path.isfile(self.FOOT_FILE) or os.path.islink(
                    self.FOOT_FILE
                ):
                    os.remove(self.FOOT_FILE)
                shutil.copyfile(theme_src, self.FOOT_FILE)
            except Exception as e:
                print(f"Error copying foot theme: {e}")

        # Apply Qt themes
        self._apply_qt_theme(new_theme)

        os.makedirs(self.WORK_PATH, exist_ok=True)
        with open(self.FILE, "w") as f:
            f.write(new_theme)

        msg = f"Trocando para o tema {new_theme}"
        subprocess.run(["notify-send", msg])
        return msg

    def _update_ini_setting(self, filepath: str, section: str, key: str, value: str):
        try:
            os.makedirs(os.path.dirname(filepath), exist_ok=True)
            config = configparser.ConfigParser(interpolation=None)
            config.optionxform = str
            if os.path.isfile(filepath):
                config.read(filepath)
            if section not in config:
                config[section] = {}
            config[section][key] = value
            with open(filepath, "w") as f:
                config.write(f)
        except Exception as e:
            print(f"Error updating INI file {filepath}: {e}")

    def _apply_qt_theme(self, new_theme: str):
        qt_style = "adwaita-dark" if new_theme == "dark" else "adwaita"
        kv_theme = "KvAdwaitaDark" if new_theme == "dark" else "KvAdwaita"
        kde_scheme = "BreezeDark" if new_theme == "dark" else "BreezeLight"

        for conf_dir in ["~/.config/qt5ct", "~/.config/qt6ct"]:
            tool_name = os.path.basename(conf_dir)
            conf_dir_expanded = os.path.expanduser(conf_dir)
            conf_path = os.path.join(conf_dir_expanded, f"{tool_name}.conf")
            if os.path.isdir(conf_dir_expanded) or os.path.isfile(conf_path) or shutil.which(tool_name):
                self._update_ini_setting(conf_path, "Appearance", "style", qt_style)

        kv_dir = os.path.expanduser("~/.config/Kvantum")
        kv_path = os.path.join(kv_dir, "kvantum.kvconfig")
        if os.path.isdir(kv_dir) or os.path.isfile(kv_path) or shutil.which("kvantummanager"):
            self._update_ini_setting(kv_path, "General", "theme", kv_theme)
            if shutil.which("kvantummanager"):
                try:
                    subprocess.run(
                        ["kvantummanager", "--set", kv_theme],
                        stderr=subprocess.DEVNULL,
                        stdout=subprocess.DEVNULL,
                    )
                except Exception as e:
                    print(f"Error calling kvantummanager: {e}")

        kde_path = os.path.expanduser("~/.config/kdeglobals")
        if os.path.isfile(kde_path) or shutil.which("plasma-apply-colorscheme"):
            self._update_ini_setting(kde_path, "General", "ColorScheme", kde_scheme)
            if shutil.which("plasma-apply-colorscheme"):
                try:
                    subprocess.run(
                        ["plasma-apply-colorscheme", kde_scheme],
                        stderr=subprocess.DEVNULL,
                        stdout=subprocess.DEVNULL,
                    )
                except Exception as e:
                    print(f"Error calling plasma-apply-colorscheme: {e}")
