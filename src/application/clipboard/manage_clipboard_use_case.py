from typing import Optional
from domain.clipboard.repositories import IClipboardRepository
from domain.clipboard.entities import ClipboardItem


class ManageClipboardUseCase:
    def __init__(self, repository: IClipboardRepository):
        self.repository = repository

    def execute(self, action: Optional[str] = None) -> bool:
        # Garante que o serviço de clipboard está em execução
        self.repository.ensure_daemon_running()

        if action == "clear":
            return self.repository.clear_history()

        if action in ("pin", "favorites"):
            return self._handle_pin_manager()

        items = self.repository.get_clipboard_items()
        if not items:
            print("Nenhum item encontrado no histórico do clipboard.")
            return False

        selected = self.repository.launch_menu(items, prompt="Clipboard...")
        if not selected:
            return False

        if selected.is_action:
            if selected.action_type == "clear":
                return self.repository.clear_history()
            elif selected.action_type == "manage_favorites":
                return self._handle_pin_manager()
            return False

        return self.repository.copy_to_clipboard(selected)

    def _handle_pin_manager(self) -> bool:
        items = self.repository.get_clipboard_items()
        pin_items = []
        for item in items:
            if item.is_action:
                continue
            if item.is_favorite:
                pin_items.append(
                    ClipboardItem(
                        id=item.id,
                        text=f"❌ [Remover dos Favoritos] {item.text}",
                        raw_preview=item.raw_preview,
                        is_image=item.is_image,
                        image_path=item.image_path,
                        is_favorite=True,
                        is_action=True,
                        action_type="unpin",
                    )
                )
            else:
                pin_items.append(
                    ClipboardItem(
                        id=item.id,
                        text=f"📌 [Fixar nos Favoritos] {item.text}",
                        raw_preview=item.raw_preview,
                        is_image=item.is_image,
                        image_path=item.image_path,
                        is_favorite=False,
                        is_action=True,
                        action_type="pin",
                    )
                )

        if not pin_items:
            print("Nenhum item disponível para fixar/desfixar.")
            return False

        selected = self.repository.launch_menu(
            pin_items, prompt="Gerenciar Favoritos..."
        )
        if not selected:
            return False

        if selected.action_type == "pin":
            clean_text = selected.text.replace("📌 [Fixar nos Favoritos] ", "")
            orig_item = ClipboardItem(
                id=selected.id,
                text=clean_text,
                raw_preview=selected.raw_preview,
                is_image=selected.is_image,
                image_path=selected.image_path,
                is_favorite=True,
            )
            return self.repository.add_favorite(orig_item)
        elif selected.action_type == "unpin":
            return self.repository.remove_favorite(selected.id)

        return False
