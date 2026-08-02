from typing import Optional
from domain.menu.repositories import IMenuRepository


class ShowMenuUseCase:
    def __init__(self, repository: IMenuRepository):
        self.repository = repository

    def execute(self, category_filter: Optional[str] = None) -> bool:
        items = self.repository.get_menu_items(category_filter=category_filter)
        if not items:
            print("Nenhum item encontrado no menu.")
            return False

        prompt = f"Buscar em [{category_filter}]..." if category_filter else "Buscar aplicativo ou comando..."
        selected = self.repository.launch_menu(items=items, prompt=prompt)

        if not selected:
            return False

        self.repository.execute_item(selected)
        return True

