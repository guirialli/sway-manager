from typing import Callable
from PySide6.QtWidgets import QWidget, QApplication
import sys

QT_QPA_PLATFORM = "wayland"


class ApplicationFactory:
    @classmethod
    def buildWidget(
        cls,
        fn_create_widget: Callable[[], QWidget],
        desktop_file_name: str | None = None,
    ):
        app = QApplication(sys.argv)
        if desktop_file_name:
            QApplication.setDesktopFileName(desktop_file_name)
        widget = fn_create_widget()
        widget.show()
        sys.exit(app.exec())
