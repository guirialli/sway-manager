from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QKeyEvent
from PySide6.QtWidgets import QWidget, QHBoxLayout, QMessageBox, QPushButton
from domain.display.entities import DisplaySwitchType
from infrastructure.display.sway_display_repository import SwayDisplayRepository
from application.display.switch_display_mode_use_case import SwitchDisplayModeUseCase
import presentation.gui.styles as styles


class MonitorSwapWindow(QWidget):
    def __init__(self, timeout: int = 15):
        super().__init__()
        self.winTitle = "SwayDisplaySwitcher"
        self.timeout = timeout
        self.use_case = SwitchDisplayModeUseCase(SwayDisplayRepository())

        self.setWindowTitle(self.winTitle)
        self.setWindowFlags(
            Qt.WindowType.FramelessWindowHint | Qt.WindowType.WindowStaysOnTopHint
        )
        self.setStyleSheet(
            f"""
                {styles.QWidget()}
                {styles.QPushButton()}
            """
        )

        self.initUI()

    def initUI(self):
        layout = QHBoxLayout()

        monitores = self.use_case.get_monitors()
        btn_pc = QPushButton(f"🖥️\n{monitores.interno}")
        btn_swap = QPushButton(f"🖥️\n{monitores.externo}")
        btn_extend = QPushButton("🖥️+🖥️\nEstender")
        btn_duplicate = QPushButton("🖥️=🖥️\nDuplicar")

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

        layout.addWidget(btn_pc)
        layout.addWidget(btn_swap)
        layout.addWidget(btn_extend)
        layout.addWidget(btn_duplicate)

        self.setLayout(layout)

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
