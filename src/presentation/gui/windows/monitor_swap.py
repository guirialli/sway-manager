from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QKeyEvent
from PySide6.QtWidgets import QWidget, QHBoxLayout, QVBoxLayout, QMessageBox, QPushButton, QLabel, QFrame
from domain.display.entities import DisplaySwitchType
from infrastructure.display.sway_display_repository import SwayDisplayRepository
from application.display.switch_display_mode_use_case import SwitchDisplayModeUseCase
import presentation.gui.styles as styles


class MonitorSwapWindow(QWidget):
    def __init__(self, timeout: int = 15, mode: str = "dark"):
        super().__init__()
        self.winTitle = "SwayDisplaySwitcher"
        self.timeout = timeout
        self.mode = mode
        self.use_case = SwitchDisplayModeUseCase(SwayDisplayRepository())

        self.setWindowTitle(self.winTitle)
        self.setWindowFlags(
            Qt.WindowType.FramelessWindowHint | Qt.WindowType.WindowStaysOnTopHint
        )
        self.setAttribute(Qt.WA_TranslucentBackground)

        c = styles.get_colors(self.mode)
        self.setStyleSheet(f"""
            QWidget#cardContainer {{
                background-color: {c['osd_bg']};
                border: 1px solid {c['osd_border']};
                border-radius: 16px;
            }}
            QLabel {{
                color: {c['text_primary']};
                font-family: {styles.FONT_FAMILY};
                background: transparent;
            }}
            QPushButton {{
                background-color: {c['button_bg']};
                border: 1px solid {c['button_border']};
                border-radius: 10px;
                padding: 16px;
                font-size: 13px;
                font-weight: 600;
                color: {c['text_primary']};
                font-family: {styles.FONT_FAMILY};
                min-width: 110px;
                min-height: 80px;
            }}
            QPushButton:hover {{
                background-color: {c['accent']};
                border-color: {c['accent']};
                color: {c['accent_text']};
            }}
        """)

        self.initUI()

    def initUI(self):
        container = QFrame()
        container.setObjectName("cardContainer")
        
        main_layout = QVBoxLayout()
        main_layout.setContentsMargins(20, 16, 20, 20)
        main_layout.setSpacing(14)
        container.setLayout(main_layout)

        header_lbl = QLabel("Disposição de Telas")
        header_lbl.setStyleSheet("font-size: 14px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: #8E8E93;")
        header_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
        main_layout.addWidget(header_lbl)

        btn_layout = QHBoxLayout()
        btn_layout.setSpacing(12)

        monitores = self.use_case.get_monitors()
        btn_pc = QPushButton(f"💻\n\n{monitores.interno}")
        btn_swap = QPushButton(f"🖥️\n\n{monitores.externo}")
        btn_extend = QPushButton("🖥️ 🖥️\n\nEstender")
        btn_duplicate = QPushButton("🖥️ = 🖥️\n\nDuplicar")

        btn_pc.clicked.connect(
            lambda: self.apply_config(DisplaySwitchType.PC_ONLY)
        )
        btn_swap.clicked.connect(
            lambda: self.apply_config(DisplaySwitchType.MONITOR_ONLY)
        )
        btn_extend.clicked.connect(
            lambda: self.apply_config(DisplaySwitchType.EXTEND)
        )
        btn_duplicate.clicked.connect(
            lambda: self.apply_config(DisplaySwitchType.DUPLICATE)
        )

        btn_layout.addWidget(btn_pc)
        btn_layout.addWidget(btn_swap)
        btn_layout.addWidget(btn_extend)
        btn_layout.addWidget(btn_duplicate)

        main_layout.addLayout(btn_layout)

        root_layout = QVBoxLayout()
        root_layout.setContentsMargins(0, 0, 0, 0)
        root_layout.addWidget(container)
        self.setLayout(root_layout)

    def apply_config(self, mode: DisplaySwitchType):
        try:
            self.use_case.execute(mode)
            self.close()
            self.confirmar_ou_reverter()
        except Exception as e:
            self.close()
            msg = QMessageBox()
            msg.setIcon(QMessageBox.Icon.Information)
            msg.setWindowTitle(self.winTitle)
            msg.setText(str(e))
            msg.setStandardButtons(QMessageBox.StandardButton.Ok)
            msg.exec()

    def confirmar_ou_reverter(self):
        msg_box = QMessageBox()
        msg_box.setWindowTitle("Confirmação de Mudança")

        msg_box.addButton("Manter", QMessageBox.ButtonRole.AcceptRole)
        btn_reverter = msg_box.addButton("Reverter", QMessageBox.ButtonRole.RejectRole)
        msg_box.setText(
            f"A configuração foi aplicada.\nVoltando ao normal em {self.timeout} segundos..."
        )

        timer = QTimer()
        timer.timeout.connect(lambda: self.atualizar_timer(msg_box, timer))
        timer.start(1000)
        msg_box.exec()
        timer.stop()

        if msg_box.clickedButton() == btn_reverter or self.timeout <= 0:
            self.use_case.recarregar_sway()

    def atualizar_timer(self, msg_box: QMessageBox, timer: QTimer):
        self.timeout -= 1
        msg_box.setText(
            f"A configuração foi aplicada.\nVoltando ao normal em {self.timeout} segundos..."
        )

        if self.timeout <= 0:
            timer.stop()
            msg_box.reject()

    def keyPressEvent(self, event: QKeyEvent, /) -> None:
        if event.key() == Qt.Key.Key_Escape:
            self.close()
        else:
            super().keyPressEvent(event)
