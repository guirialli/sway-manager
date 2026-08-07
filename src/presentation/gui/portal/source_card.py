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
        if self._selected:
            background = "qlineargradient(x1: 0, y1: 0, x2: 1, y2: 1, stop: 0 #7c3aed, stop: 1 #a855f7)"
            foreground = "#ffffff"
            details_color = "rgba(255, 255, 255, 0.9)"
            border = "#a78bfa"
            shadow = "#5b21b6"
        else:
            background = "#1e1b2e"
            foreground = "#f8fafc"
            details_color = "rgba(248, 250, 252, 0.65)"
            border = "#3d3452"
            shadow = "transparent"

        self.setStyleSheet(
            f"""
            QFrame#sourceCard {{
                background: {background};
                border: 1px solid {border};
                border-radius: 12px;
                padding: 2px;
            }}
            QFrame#sourceCard:hover {{
                background: qlineargradient(
                    x1: 0, y1: 0, x2: 1, y2: 1,
                    stop: 0 #262236, stop: 1 #2d2640
                );
                border: 1px solid #5b4e78;
            }}
            QFrame#sourceCard:focus {{
                border: 2px solid #8b5cf6;
            }}
            QFrame#sourceCard:hover:focus {{
                border: 2px solid #a78bfa;
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
