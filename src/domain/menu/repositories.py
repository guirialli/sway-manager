from abc import ABC, abstractmethod
from typing import List, Optional
from domain.menu.entities import MenuItem


class IMenuRepository(ABC):
    @abstractmethod
    def get_menu_items(self, category_filter: Optional[str] = None) -> List[MenuItem]:
        pass

    @abstractmethod
    def launch_menu(self, items: List[MenuItem], prompt: str = "Buscar...") -> Optional[MenuItem]:
        pass

    @abstractmethod
    def execute_item(self, item: MenuItem) -> None:
        pass
