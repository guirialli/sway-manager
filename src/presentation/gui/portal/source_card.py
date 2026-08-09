from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont, QPalette
from PySide6.QtWidgets import QFrame, QLabel, QSizePolicy, QVBoxLayout

from portal.models import PortalSource


class SourceCard(QFrame):
    """Native-theme selectable row representing a shareable source."""

    clicked = Signal()
    activated = Signal()
    next_requested = Signal()
    previous_requested = Signal()
    def __init__(self, source: PortalSource):
        super().__init__()
        self.source = source
        self._selected = False

        self.setObjectName("sourceCard")
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setFrameShape(QFrame.Shape.StyledPanel)
        self.setMinimumHeight(92)
        self.setSizePolicy(
            QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Minimum
        )
        self.setAccessibleName(
            f"{source.label}. {source.details.replace(chr(10), ', ')}"
        )

        self._build_ui()
        self._update_style()

    def _build_ui(self) -> None:
        layout = QVBoxLayout()
        layout.setContentsMargins(16, 14, 16, 14)
        layout.setSpacing(4)

        self._title = QLabel(self.source.label)
        self._title.setWordWrap(True)
        title_font = QFont(self._title.font())
        title_font.setBold(True)
        self._title.setFont(title_font)

        self._details = QLabel(self.source.details)
        self._details.setWordWrap(True)

        layout.addWidget(self._title)
        layout.addWidget(self._details)
        self.setLayout(layout)

    def set_selected(self, selected: bool) -> None:
        self._selected = selected
        self._update_style()
        if selected:
            self.setFocus()

    def is_selected(self) -> bool:
        return self._selected

    def _update_style(self) -> None:
        from presentation.gui.styles import get_colors
        c = get_colors("dark")
        if self._selected:
            background = c["selection_bg"]
            foreground = c["accent_text"]
            details_color = "rgba(255, 255, 255, 0.9)"
            border = c["accent_hover"]
        else:
            background = c["card_bg"]
            foreground = c["text_primary"]
            details_color = c["text_secondary"]
            border = c["card_border"]

        self.setStyleSheet(
            f"""
            QFrame#sourceCard {{
                background-color: {background};
                border: 1px solid {border};
                border-radius: 10px;
                padding: 2px;
            }}
            QFrame#sourceCard:hover {{
                background-color: {c['button_hover']};
                border: 1px solid {c['accent']};
            }}
            QFrame#sourceCard:focus {{
                border: 2px solid {c['accent']};
            }}
            QFrame#sourceCard:hover:focus {{
                border: 2px solid {c['accent_hover']};
            }}
            QLabel {{
                color: {foreground};
                background: transparent;
            }}
            """
        )
        if hasattr(self, "_details"):
            self._details.setStyleSheet(
                f"color: {details_color}; background: transparent;"
            )
    def mouseReleaseEvent(self, event) -> None:
        clicked = (
            event.button() == Qt.MouseButton.LeftButton
            and self.rect().contains(event.position().toPoint())
        )
        super().mouseReleaseEvent(event)
        if clicked:
            self.clicked.emit()

    def mouseDoubleClickEvent(self, event) -> None:
        if event.button() == Qt.MouseButton.LeftButton:
            self.activated.emit()
            event.accept()
            return
        super().mouseDoubleClickEvent(event)

    def keyPressEvent(self, event) -> None:
        if event.key() in (Qt.Key.Key_Return, Qt.Key.Key_Enter, Qt.Key.Key_Space):
            self.activated.emit()
            event.accept()
            return
        if event.key() in (Qt.Key.Key_Right, Qt.Key.Key_Down):
            self.next_requested.emit()
            event.accept()
            return
        if event.key() in (Qt.Key.Key_Left, Qt.Key.Key_Up):
            self.previous_requested.emit()
            event.accept()
            return
        super().keyPressEvent(event)
