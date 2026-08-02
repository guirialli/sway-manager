from abc import ABC, abstractmethod
from typing import List, Optional
from domain.clipboard.entities import ClipboardItem


class IClipboardRepository(ABC):
    @abstractmethod
    def ensure_daemon_running(self) -> bool:
        """Garante que o daemon de escuta do clipboard (wl-paste --watch cliphist store) está rodando."""
        pass

    @abstractmethod
    def get_clipboard_items(self) -> List[ClipboardItem]:
        """Obtém todos os itens do clipboard (favoritos no topo seguidos pelo histórico cliphist)."""
        pass

    @abstractmethod
    def copy_to_clipboard(self, item: ClipboardItem) -> bool:
        """Decodifica e copia o item para a área de transferência do Wayland (wl-copy)."""
        pass

    @abstractmethod
    def clear_history(self, keep_favorites: bool = True) -> bool:
        """Limpa o histórico do cliphist."""
        pass

    @abstractmethod
    def get_favorites(self) -> List[ClipboardItem]:
        """Retorna a lista de itens favoritos fixados."""
        pass

    @abstractmethod
    def add_favorite(self, item: ClipboardItem) -> bool:
        """Fixa um item como favorito."""
        pass

    @abstractmethod
    def remove_favorite(self, item_id: str) -> bool:
        """Remove um item dos favoritos."""
        pass

    @abstractmethod
    def launch_menu(
        self, items: List[ClipboardItem], prompt: str = "Clipboard..."
    ) -> Optional[ClipboardItem]:
        """Exibe a interface gráfica do Wofi para o usuário selecionar um item ou ação."""
        pass
