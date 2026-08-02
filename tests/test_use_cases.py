import unittest
from unittest.mock import MagicMock
from domain.theme.entities import LightDMSettings, AppearanceSettings
from application.theme.update_lightdm_use_case import UpdateLightDMUseCase
from application.theme.manage_appearance_use_case import ManageAppearanceUseCase


class TestUseCases(unittest.TestCase):
    def test_update_lightdm_use_case(self):
        mock_repo = MagicMock()
        mock_repo.get_settings.return_value = LightDMSettings(theme_name="Adwaita-dark")
        mock_repo.save_settings.return_value = True
        mock_repo.get_available_users.return_value = ["guilherme"]
        mock_repo.get_available_sessions.return_value = ["sway"]

        use_case = UpdateLightDMUseCase(mock_repo)

        settings = use_case.get_settings()
        self.assertEqual(settings.theme_name, "Adwaita-dark")

        success = use_case.execute(settings, image_source_path="/tmp/bg.jpg")
        self.assertTrue(success)
        mock_repo.save_settings.assert_called_with(settings, "/tmp/bg.jpg")

        users = use_case.get_available_users()
        self.assertEqual(users, ["guilherme"])

        sessions = use_case.get_available_sessions()
        self.assertEqual(sessions, ["sway"])

    def test_manage_appearance_use_case(self):
        mock_repo = MagicMock()
        mock_repo.get_appearance_settings.return_value = AppearanceSettings(
            gtk_theme="Orchis-Dark"
        )
        mock_repo.apply_appearance.return_value = True

        use_case = ManageAppearanceUseCase(mock_repo)

        settings = use_case.get_settings()
        self.assertEqual(settings.gtk_theme, "Orchis-Dark")

        success = use_case.apply(settings)
        self.assertTrue(success)
        mock_repo.apply_appearance.assert_called_with(settings)


if __name__ == "__main__":
    unittest.main()
