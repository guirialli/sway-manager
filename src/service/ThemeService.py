import os
import shutil
import subprocess
import json


class ThemeService:
    WORK_PATH = os.path.expanduser("~/.config/theme_toggle")
    FILE = os.path.join(WORK_PATH, "theme")
    FOOT_DIR = os.path.expanduser("~/.config/foot")
    FOOT_FILE = os.path.join(FOOT_DIR, "foot.ini")

    @classmethod
    def get_current_theme(cls) -> str:
        if os.path.isfile(cls.FILE):
            try:
                with open(cls.FILE, "r") as f:
                    return f.read().strip()
            except Exception:
                pass
        return "light"

    @classmethod
    def status(cls):
        theme = cls.get_current_theme()
        if theme == "light":
            print(
                json.dumps(
                    {
                        "text": "☀️",
                        "tooltip": "Tema Light",
                        "class": "theme-light",
                    }
                )
            )
        else:
            print(
                json.dumps(
                    {
                        "text": "🌕",
                        "tooltip": "Tema Dark",
                        "class": "theme-dark",
                    }
                )
            )

    @classmethod
    def toggle(cls):
        current = cls.get_current_theme()
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
        theme_src = os.path.join(cls.FOOT_DIR, "themes", foot_theme)
        if os.path.isfile(theme_src):
            try:
                if os.path.isfile(cls.FOOT_FILE) or os.path.islink(
                    cls.FOOT_FILE
                ):
                    os.remove(cls.FOOT_FILE)
                shutil.copyfile(theme_src, cls.FOOT_FILE)
            except Exception as e:
                print(f"Error copying foot theme: {e}")

        os.makedirs(cls.WORK_PATH, exist_ok=True)
        with open(cls.FILE, "w") as f:
            f.write(new_theme)

        msg = f"Trocando para o tema {new_theme}"
        subprocess.run(["notify-send", msg])
        print(msg)
