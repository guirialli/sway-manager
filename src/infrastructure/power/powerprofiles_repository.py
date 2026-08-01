import subprocess
from typing import Optional
from domain.power.entities import PowerProfileState
from domain.power.repositories import IPowerProfileRepository
from domain.notification.entities import Notification
from domain.notification.repositories import INotificationRepository
from infrastructure.notification.desktop_notification_repository import (
    DesktopNotificationRepository,
)


class PowerProfilesRepository(IPowerProfileRepository):
    PROFILES = ["power-saver", "balanced", "performance"]

    def __init__(self, notification_repo: Optional[INotificationRepository] = None):
        self.notification_repo = notification_repo or DesktopNotificationRepository()

    def get_state(self) -> PowerProfileState:
        try:
            res = subprocess.run(
                ["powerprofilesctl", "get"], capture_output=True, text=True
            )
            val = res.stdout.strip()
            if val in self.PROFILES:
                return PowerProfileState(active_profile=val)
        except Exception:
            pass
        return PowerProfileState(active_profile="balanced")

    def set_profile(self, target: str) -> str:
        subprocess.run(["powerprofilesctl", "set", target])
        msg_name = PowerProfileState.PROFILE_NAMES.get(target, target)
        self.notification_repo.notify(Notification(title=f"{msg_name} setado"))
        return f"Power profile set to: {target}"
