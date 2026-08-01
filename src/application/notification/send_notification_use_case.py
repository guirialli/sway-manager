from typing import Optional
from domain.notification.entities import Notification
from domain.notification.repositories import INotificationRepository


class SendNotificationUseCase:
    def __init__(self, repository: INotificationRepository):
        self.repository = repository

    def execute(
        self,
        title: str,
        message: str = "",
        urgency: str = "normal",
        icon: Optional[str] = None,
    ) -> None:
        notification = Notification(
            title=title, message=message, urgency=urgency, icon=icon
        )
        self.repository.notify(notification)
