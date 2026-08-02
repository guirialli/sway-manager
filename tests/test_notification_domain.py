import unittest
from unittest.mock import patch, MagicMock
from domain.notification.entities import Notification
from domain.notification.repositories import INotificationRepository
from infrastructure.notification.desktop_notification_repository import (
    DesktopNotificationRepository,
)
from application.notification.send_notification_use_case import (
    SendNotificationUseCase,
)


class MockNotificationRepository(INotificationRepository):
    def __init__(self):
        self.sent_notifications = []

    def notify(self, notification: Notification) -> None:
        self.sent_notifications.append(notification)


class TestNotificationDomain(unittest.TestCase):
    def test_notification_entity_defaults(self):
        n = Notification(title="Teste")
        self.assertEqual(n.title, "Teste")
        self.assertEqual(n.message, "")
        self.assertEqual(n.urgency, "normal")
        self.assertIsNone(n.icon)

    def test_notification_entity_custom(self):
        n = Notification(
            title="Alerta", message="Mensagem", urgency="critical", icon="dialog-warning"
        )
        self.assertEqual(n.title, "Alerta")
        self.assertEqual(n.message, "Mensagem")
        self.assertEqual(n.urgency, "critical")
        self.assertEqual(n.icon, "dialog-warning")

    @patch("subprocess.run")
    def test_desktop_notification_repository(self, mock_run):
        repo = DesktopNotificationRepository()

        # Test normal notification
        n1 = Notification(title="Titulo 1", message="Corpo 1")
        repo.notify(n1)
        mock_run.assert_called_with(
            ["notify-send", "Titulo 1", "Corpo 1"], check=False
        )

        # Test critical notification with icon
        n2 = Notification(
            title="Critico", message="Erro grave", urgency="critical", icon="error"
        )
        repo.notify(n2)
        mock_run.assert_called_with(
            ["notify-send", "-u", "critical", "-i", "error", "Critico", "Erro grave"],
            check=False,
        )

    def test_send_notification_use_case(self):
        mock_repo = MockNotificationRepository()
        use_case = SendNotificationUseCase(mock_repo)

        use_case.execute(title="Aviso", message="Processo concluído", urgency="normal")

        self.assertEqual(len(mock_repo.sent_notifications), 1)
        sent = mock_repo.sent_notifications[0]
        self.assertEqual(sent.title, "Aviso")
        self.assertEqual(sent.message, "Processo concluído")
        self.assertEqual(sent.urgency, "normal")


if __name__ == "__main__":
    unittest.main()
