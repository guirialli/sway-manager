from interface.IMonitor import ISwitcherUI, MonitoresSway, DisplaySwitchType
from service.SwayService import SwayService
from PySide6.QtWidgets import QMessageBox
from PySide6.QtCore import QTimer
import subprocess


class DisplaySwitcherController:
    def __init__(
        self, widget: ISwitcherUI, windowTitle: str, timeout: int = 15
    ) -> None:
        self.widget = widget
        self.windowTitle = windowTitle
        self.timeout = timeout

    def get_monitors(self) -> MonitoresSway:
        SwayService.validar_monitores()
        monitores = SwayService.get_monitores()

        if monitores[1].startswith("eDP") or monitores[0].endswith("2"):
            return MonitoresSway(interno=monitores[1], externo=monitores[0])

        return MonitoresSway(interno=monitores[0], externo=monitores[1])

    def apply_config(self, mode: DisplaySwitchType):
        try:
            monitor = self.get_monitors()
            cmd = ""

            if DisplaySwitchType.PC_ONLY == mode:
                cmd = f"swaymsg output {monitor.interno} enable, output {monitor.externo} disable"
            elif DisplaySwitchType.MONITOR_ONLY == mode:
                cmd = f"swaymsg output {monitor.interno} disable, output {monitor.externo} enable"
            elif DisplaySwitchType.EXTEND == mode:
                cmd = f"swaymsg output {monitor.interno} enable, output {monitor.externo} enable"
            elif DisplaySwitchType.DUPLICATE == mode:
                self.widget.close()
                cmd = f"swaymsg output {monitor.interno} enable, output {monitor.externo} enable; pkill wl-mirror; wl-mirror {monitor.interno}"
                subprocess.run(cmd, shell=True, check=True)
                return

            subprocess.run(cmd, shell=True, check=True)
            print(f"Modo aplicado {mode.value}")

            self.widget.close()
            self.widget.confirmar_ou_reverter()
        except Exception as e:
            self.widget.close()
            msg = QMessageBox()
            msg.setIcon(QMessageBox.Icon.Information)
            msg.setWindowTitle(self.windowTitle)
            msg.setText(e.__str__())
            msg.setStandardButtons(QMessageBox.StandardButton.Ok)
            msg.exec()

            self.widget.close()
            raise Exception("Erro: apenas um monitor econtrado!")

    def recarregar_sway(self):
        SwayService.recarregar()

    def atualizar_timer(self, msg_box: QMessageBox, timer: QTimer):
        self.timeout -= 1
        msg_box.setText(
            f"A configuração foi aplicada.\nVoltando ao normal em {self.timeout} segundos..."
        )

        if self.timeout <= 0:
            timer.stop()
            msg_box.reject()
