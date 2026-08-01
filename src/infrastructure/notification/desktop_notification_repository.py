import subprocess
from domain.notification.entities import Notification
from domain.notification.repositories import INotificationRepository


class DesktopNotificationRepository(INotificationRepository):
    def notify(self, notification: Notification) -> None:
        cmd = ["notify-send"]
        if notification.urgency and notification.urgency != "normal":
            cmd.extend(["-u", notification.urgency])
        if notification.icon:
            cmd.extend(["-i", notification.icon])

        cmd.append(notification.title)
        if notification.message:
            cmd.append(notification.message)

        try:
            subprocess.run(cmd, check=False)
        except Exception as e:
            print(f"Error sending notification: {e}")
