from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont
from PySide6.QtWidgets import (
    QDialog,
    QFrame,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QScrollArea,
    QSizePolicy,
    QStackedWidget,
    QTabBar,
    QVBoxLayout,
    QWidget,
)

from portal.models import PortalResult, PortalSource, PortalSourceType
from presentation.gui.portal.empty_state import EmptyState
from presentation.gui.portal.source_card import SourceCard


class PortalDialog(QDialog):
    """Native-theme source selector for xdg-desktop-portal-wlr."""

    source_selected = Signal(PortalResult)
    cancelled = Signal()

    def __init__(
        self,
        monitors: list[PortalSource],
        windows: list[PortalSource],
        window_sharing_reason: str | None = None,
    ) -> None:
        super().__init__()
        self.monitors = monitors
        self.windows = windows
        self.window_sharing_reason = window_sharing_reason
        self._selected_card: SourceCard | None = None
        self._cards: list[SourceCard] = []

        self.setWindowTitle("Compartilhar tela")
        self.setWindowModality(Qt.WindowModality.ApplicationModal)
        self.setWindowFlags(
            Qt.WindowType.Dialog
            | Qt.WindowType.WindowStaysOnTopHint
            | Qt.WindowType.WindowCloseButtonHint
        )
        self.resize(720, 520)
        self.setMinimumSize(480, 380)
        self._build_ui()
        self._connect_signals()
        self._apply_theme()

    def _build_ui(self) -> None:
        root = QVBoxLayout(self)
        root.setContentsMargins(24, 24, 24, 20)
        root.setSpacing(16)

        header = QLabel("Escolha o que deseja compartilhar")
        header.setObjectName("portalHeader")
        header_font = QFont(header.font())
        header_font.setBold(True)
        header_font.setPointSize(header_font.pointSize() + 5)
        header.setFont(header_font)
        root.addWidget(header)

        description = QLabel(
            "Selecione uma tela ou janela. Somente o conteúdo escolhido será "
            "compartilhado."
        )
        description.setObjectName("portalDescription")
        description.setWordWrap(True)
        description.setSizePolicy(
            QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Minimum
        )
        root.addWidget(description)

        self._tab_bar = QTabBar()
        self._tab_bar.setDocumentMode(False)
        self._tab_bar.setExpanding(False)
        self._tab_bar.setDrawBase(False)
        self._tab_bar.addTab("Telas")
        self._tab_bar.addTab("Janelas")

        tab_container = QHBoxLayout()
        tab_container.addStretch()
        tab_container.addWidget(self._tab_bar)
        tab_container.addStretch()
        root.addLayout(tab_container)

        self._stack = QStackedWidget()
        self._stack.addWidget(
            self._build_page(self.monitors, PortalSourceType.MONITOR)
        )
        self._stack.addWidget(
            self._build_page(self.windows, PortalSourceType.WINDOW)
        )
        root.addWidget(self._stack, 1)

        actions = QHBoxLayout()
        actions.addStretch()

        self._cancel_btn = QPushButton("Cancelar")
        self._share_btn = QPushButton("Compartilhar")
        self._share_btn.setDefault(True)
        self._share_btn.setEnabled(False)

        actions.addWidget(self._cancel_btn)
        actions.addWidget(self._share_btn)
        root.addLayout(actions)

    def _build_page(
        self, sources: list[PortalSource], source_type: PortalSourceType
    ) -> QWidget:
        page = QWidget()
        layout = QVBoxLayout(page)
        layout.setContentsMargins(0, 0, 0, 0)

        if not sources:
            title, description = self._empty_state_copy(source_type)
            layout.addWidget(EmptyState(title, description))
            return page

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.Shape.NoFrame)
        scroll.setHorizontalScrollBarPolicy(
            Qt.ScrollBarPolicy.ScrollBarAlwaysOff
        )

        container = QWidget()
        source_layout = QVBoxLayout(container)
        source_layout.setContentsMargins(0, 0, 0, 0)
        source_layout.setSpacing(8)

        for source in sources:
            card = SourceCard(source)
            card.clicked.connect(self._make_select_handler(card))
            card.activated.connect(self._make_activate_handler(card))
            card.next_requested.connect(self._focus_next_card)
            card.previous_requested.connect(self._focus_previous_card)
            self._cards.append(card)
            source_layout.addWidget(card)
        source_layout.addStretch()

        scroll.setWidget(container)
        layout.addWidget(scroll)
        return page

    def _empty_state_copy(
        self, source_type: PortalSourceType
    ) -> tuple[str, str]:
        if source_type == PortalSourceType.MONITOR:
            return "Nenhuma tela ativa encontrada", (
                "Verifique se há um output ativo no Sway e abra o seletor novamente."
            )
        if self.window_sharing_reason:
            return "Captura de janela indisponível", self.window_sharing_reason
        return "Nenhuma janela compartilhável encontrada", (
            "Abra uma janela compatível e inicie uma nova solicitação de "
            "compartilhamento."
        )

    def _connect_signals(self) -> None:
        self._tab_bar.currentChanged.connect(self._on_tab_changed)
        self._cancel_btn.clicked.connect(self._on_cancel)
        self._share_btn.clicked.connect(self._accept_selection)

    def _on_tab_changed(self, index: int) -> None:
        self._stack.setCurrentIndex(index)
        self._set_selected_card(None)

    def _make_select_handler(self, card: SourceCard):
        def select() -> None:
            self._set_selected_card(card)

        return select

    def _make_activate_handler(self, card: SourceCard):
        def activate() -> None:
            self._set_selected_card(card)
            self._accept_selection()

        return activate

    def _set_selected_card(self, card: SourceCard | None) -> None:
        if self._selected_card is not None and self._selected_card != card:
            self._selected_card.set_selected(False)
        self._selected_card = card
        if card is not None:
            card.set_selected(True)
        self._share_btn.setEnabled(card is not None)

    def _accept_selection(self) -> None:
        if self._selected_card is None:
            return
        source = self._selected_card.source
        self.source_selected.emit(
            PortalResult(
                source_type=source.source_type,
                id=source.id,
                raw_label=source.raw_label,
            )
        )
        self.accept()

    def _on_cancel(self) -> None:
        self.cancelled.emit()
        self.reject()

    def keyPressEvent(self, event) -> None:
        if event.key() == Qt.Key.Key_Escape:
            self._on_cancel()
            return
        if event.key() in (Qt.Key.Key_Right, Qt.Key.Key_Down):
            self._focus_next_card()
            return
        if event.key() in (Qt.Key.Key_Left, Qt.Key.Key_Up):
            self._focus_previous_card()
            return
        super().keyPressEvent(event)

    def _focus_next_card(self) -> None:
        visible_cards = self._visible_cards()
        if not visible_cards:
            return
        if self._selected_card not in visible_cards:
            self._set_selected_card(visible_cards[0])
            return
        index = visible_cards.index(self._selected_card)
        self._set_selected_card(visible_cards[(index + 1) % len(visible_cards)])

    def _focus_previous_card(self) -> None:
        visible_cards = self._visible_cards()
        if not visible_cards:
            return
        if self._selected_card not in visible_cards:
            self._set_selected_card(visible_cards[-1])
            return
        index = visible_cards.index(self._selected_card)
        self._set_selected_card(visible_cards[(index - 1) % len(visible_cards)])

    def _visible_cards(self) -> list[SourceCard]:
        source_type = (
            PortalSourceType.MONITOR
            if self._stack.currentIndex() == 0
            else PortalSourceType.WINDOW
        )
        return [
            card for card in self._cards if card.source.source_type == source_type
        ]

    def showEvent(self, event) -> None:
        super().showEvent(event)
        self._cancel_btn.setFocus()

    def _apply_theme(self) -> None:
        self.setStyleSheet(
            """
            QDialog {
                background: qradialgradient(
                    cx: 0.5, cy: 0.3, radius: 0.8,
                    stop: 0 #1a1625, stop: 1 #12101a
                );
                color: #f8fafc;
            }
            QLabel {
                color: #f8fafc;
                background: transparent;
            }
            QLabel#portalHeader {
                color: #ffffff;
                padding-bottom: 4px;
            }
            QLabel#portalDescription {
                color: rgba(248, 250, 252, 0.7);
                padding-bottom: 8px;
            }
            QTabBar {
                background: transparent;
                border: none;
            }
            QTabBar::tab {
                background: #1e1b2e;
                color: rgba(248, 250, 252, 0.75);
                border: 1px solid #3d3452;
                border-radius: 10px;
                padding: 8px 28px;
                margin-right: 8px;
                min-width: 90px;
                max-width: 140px;
                font-weight: bold;
            }
            QTabBar::tab:selected {
                background: qlineargradient(
                    x1: 0, y1: 0, x2: 1, y2: 1,
                    stop: 0 #7c3aed, stop: 1 #a855f7
                );
                color: #ffffff;
                border: 1px solid #8b5cf6;
            }
            QTabBar::tab:hover {
                background: #262236;
                color: #f8fafc;
            }
            QTabBar::tab:selected:hover {
                background: qlineargradient(
                    x1: 0, y1: 0, x2: 1, y2: 1,
                    stop: 0 #7c3aed, stop: 1 #a855f7
                );
                color: #ffffff;
            }
            QScrollArea {
                background: transparent;
                border: none;
            }
            QScrollArea > QWidget > QWidget {
                background: transparent;
            }
            QScrollBar:vertical {
                background: transparent;
                width: 8px;
                margin: 0px;
            }
            QScrollBar::handle:vertical {
                background: #4c3f6b;
                border-radius: 4px;
                min-height: 30px;
            }
            QScrollBar::handle:vertical:hover {
                background: #7c3aed;
            }
            QScrollBar::add-line:vertical,
            QScrollBar::sub-line:vertical {
                height: 0px;
            }
            QPushButton {
                background: #1e1b2e;
                color: rgba(248, 250, 252, 0.9);
                border: 1px solid #3d3452;
                border-radius: 10px;
                padding: 8px 22px;
                font-weight: bold;
                min-width: 100px;
            }
            QPushButton:hover {
                background: #262236;
                border-color: #5b4e78;
                color: #f8fafc;
            }
            QPushButton:pressed {
                background: #161421;
            }
            QPushButton:disabled {
                background: #161421;
                color: rgba(248, 250, 252, 0.35);
                border: 1px solid #2a2438;
            }
            QPushButton[default="true"] {
                background: qlineargradient(
                    x1: 0, y1: 0, x2: 1, y2: 1,
                    stop: 0 #7c3aed, stop: 1 #a855f7
                );
                color: #ffffff;
                border: 1px solid #8b5cf6;
            }
            QPushButton[default="true"]:hover {
                background: qlineargradient(
                    x1: 0, y1: 0, x2: 1, y2: 1,
                    stop: 0 #8b5cf6, stop: 1 #c084fc
                );
                border-color: #a78bfa;
            }
            QPushButton[default="true"]:pressed {
                background: qlineargradient(
                    x1: 0, y1: 0, x2: 1, y2: 1,
                    stop: 0 #6d28d9, stop: 1 #9333ea
                );
            }
            QPushButton[default="true"]:disabled {
                background: #2d1f4a;
                color: rgba(248, 250, 252, 0.5);
                border: 1px solid #3d2b61;
            }
            QFrame#emptyState {
                background: #1e1b2e;
                border: 1px solid #3d3452;
                border-radius: 12px;
            }
            QLabel#emptyStateDesc {
                color: rgba(248, 250, 252, 0.65);
            }
            """
        )
