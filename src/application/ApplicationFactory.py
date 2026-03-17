from typing import Callable
from PySide6.QtWidgets import QWidget, QApplication
import sys

QT_QPA_PLATFORM = "wayland"


class ApplicationFactory:
    @classmethod
    def buildWidget(cls, fn_create_widget: Callable[[], QWidget]):
        app = QApplication(sys.argv)
        widget = fn_create_widget()
        widget.show()
        sys.exit(app.exec())
