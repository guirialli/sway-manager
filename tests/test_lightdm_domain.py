import unittest
from domain.theme.entities import LightDMSettings


class TestLightDMDomain(unittest.TestCase):
    def test_lightdm_settings_defaults(self):
        settings = LightDMSettings()
        self.assertEqual(settings.background, "/usr/share/backgrounds/lightdm-wallpaper.jpg")
        self.assertEqual(settings.theme_name, "Adwaita-dark")
        self.assertEqual(settings.icon_theme_name, "Adwaita")
        self.assertEqual(settings.font_name, "Sans 11")
        self.assertEqual(settings.cursor_theme_name, "Bibata-Modern-Ice")
        self.assertEqual(settings.cursor_theme_size, "24")
        self.assertEqual(settings.xft_antialias, "true")
        self.assertEqual(settings.clock_format, "%a, %d %b %H:%M")
        self.assertEqual(settings.autologin_user, "")
        self.assertEqual(settings.user_session, "sway")

    def test_to_dict_and_from_dict_roundtrip(self):
        original_dict = {
            "background": "/path/to/custom_bg.png",
            "theme-name": "Orchis-Dark",
            "icon-theme-name": "Papirus",
            "font-name": "Inter 12",
            "cursor-theme-name": "Adwaita",
            "cursor-theme-size": "32",
            "xft-antialias": "false",
            "xft-dpi": "96",
            "xft-hintstyle": "hintfull",
            "xft-rgba": "rgb",
            "clock-format": "%H:%M",
            "position": "50%,50%",
            "active-monitor": "0",
            "screensaver-timeout": "120",
            "indicators": "~host;~clock;~power",
            "draw-user-backgrounds": "true",
            "draw-grid": "true",
            "hide-user-image": "true",
            "default-user-image": "/avatar.png",
            "autologin-user": "guilherme",
            "autologin-user-timeout": "5",
            "autologin-session": "sway",
            "user-session": "sway",
            "greeter-show-manual-login": "true",
            "greeter-hide-users": "true",
            "allow-guest": "false",
        }

        settings = LightDMSettings.from_dict(original_dict)
        self.assertEqual(settings.background, "/path/to/custom_bg.png")
        self.assertEqual(settings.theme_name, "Orchis-Dark")
        self.assertEqual(settings.autologin_user, "guilherme")
        self.assertEqual(settings.autologin_user_timeout, "5")
        self.assertEqual(settings.draw_grid, "true")

        exported_dict = settings.to_dict()
        self.assertEqual(exported_dict, original_dict)


if __name__ == "__main__":
    unittest.main()
