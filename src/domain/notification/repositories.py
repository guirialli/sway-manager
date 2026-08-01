from abc import ABC, abstractmethod
from domain.notification.entities import Notification


class INotificationRepository(ABC):
    @abstractmethod
    def notify(self, notification: Notification) -> None:
        pass
