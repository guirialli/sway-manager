from controller.MonitorSwapController import DisplaySwitcherController
from service.SwayService import SwayService
from interface.IMonitor import ISwitcherUI, DisplaySwitchType
from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QKeyEvent
from PySide6.QtWidgets import QHBoxLayout, QMessageBox, QPushButton
import ui.Styles as styles
import time


class MonitorCmd:
    def __init__(self, arg: str) -> None:
        self.arg = arg.lower().strip()

        if self.arg == "-d":
            self.monitorar_monitor()

    def monitorar_monitor(self):
        monitores_anterior = SwayService.get_monitores()

        while True:
            monitores_atual = SwayService.get_monitores()
            if len(monitores_anterior) != len(monitores_atual):
                monitores_anterior = monitores_atual
                SwayService.recarregar()

            time.sleep(2)


class MonitorSwap(ISwitcherUI):
    def __init__(self):
        super().__init__()
        self.winTitle = "SwayDisplaySwitcher"
        self.controller = DisplaySwitcherController(self, self.winTitle)

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

        monitores = self.controller.get_monitors()
        btn_pc = QPushButton(f"🖥️\n{monitores.interno}")
        btn_swap = QPushButton(f"🖥️\n{monitores.externo}")
        btn_extend = QPushButton("🖥️+🖥️\nEstender")
        btn_duplicate = QPushButton("🖥️=🖥️\nDuplicar")

        btn_pc.clicked.connect(
            lambda: self.controller.apply_config(DisplaySwitchType.PC_ONLY)
        )
        btn_swap.clicked.connect(
            lambda: self.controller.apply_config(DisplaySwitchType.MONITOR_ONLY)
        )
        btn_extend.clicked.connect(
            lambda: self.controller.apply_config(DisplaySwitchType.EXTEND)
        )
        btn_duplicate.clicked.connect(
            lambda: self.controller.apply_config(DisplaySwitchType.DUPLICATE)
        )

        layout.addWidget(btn_pc)
        layout.addWidget(btn_swap)
        layout.addWidget(btn_extend)
        layout.addWidget(btn_duplicate)

        self.setLayout(layout)

    def confirmar_ou_reverter(self):
        msg_box = QMessageBox()
        msg_box.setWindowTitle("Confirmação de Mudança")

        msg_box.addButton("Manter", QMessageBox.ButtonRole.AcceptRole)
        btn_reverter = msg_box.addButton("Reverter", QMessageBox.ButtonRole.RejectRole)
        msg_box.setText(
            f"A configuração foi aplicada.\nVoltando ao normal em {self.controller.timeout} segundos..."
        )

        timer = QTimer()
        timer.timeout.connect(lambda: self.controller.atualizar_timer(msg_box, timer))
        timer.start(1000)
        msg_box.exec()
        timer.stop()

        if msg_box.clickedButton() == btn_reverter or self.controller.timeout <= 0:
            self.controller.recarregar_sway()

    def keyPressEvent(self, event: QKeyEvent, /) -> None:
        if event.key() == Qt.Key.Key_Escape:
            self.close()
        else:
            super().keyPressEvent(event)
