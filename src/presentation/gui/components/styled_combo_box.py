from typing import List, Optional
from PySide6.QtWidgets import QComboBox, QListView


class ThemeComboBox(QComboBox):
    """
    Componente reutilizável de QComboBox para seleção de temas GTK,
    ícones, cursores e fontes no SwayManager.
    """
    def __init__(
        self,
        items: Optional[List[str]] = None,
        current_text: Optional[str] = None,
        editable: bool = False,
        parent=None,
    ):
        super().__init__(parent)
        self.setView(QListView())
        self.setEditable(editable)
        if items:
            self.addItems(items)
        if current_text:
            self.set_current_text_safe(current_text)

    def set_current_text_safe(self, text: str):
        if not text:
            return
        idx = self.findText(text)
        if idx >= 0:
            self.setCurrentIndex(idx)
        else:
            if self.isEditable():
                self.setCurrentText(text)
            else:
                for i in range(self.count()):
                    if self.itemText(i) in text or text in self.itemText(i):
                        self.setCurrentIndex(i)
                        return
                self.setCurrentText(text)
