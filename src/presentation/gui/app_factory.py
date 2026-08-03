from typing import Callable
from PySide6.QtWidgets import QWidget, QApplication
import sys

QT_QPA_PLATFORM = "wayland"


class ApplicationFactory:
    _active_widgets: list[QWidget] = []

    @classmethod
    def buildWidget(
        cls,
        fn_create_widget: Callable[[], QWidget],
        desktop_file_name: str | None = None,
    ):
        app = QApplication.instance()
        is_standalone = app is None

        if is_standalone:
            app = QApplication(sys.argv)

        if desktop_file_name:
            QApplication.setDesktopFileName(desktop_file_name)

        try:
            widget = fn_create_widget()
            cls._active_widgets.append(widget)
            widget.destroyed.connect(
                lambda: cls._active_widgets.remove(widget)
                if widget in cls._active_widgets
                else None
            )
            widget.show()
            widget.raise_()
            widget.activateWindow()
        except Exception as ex:
            import traceback
            from infrastructure.logging.async_logger import get_logger
            get_logger().error(f"Erro ao construir interface gráfica Qt: {ex}\n{traceback.format_exc().strip()}")
            if is_standalone:
                sys.exit(1)
            return

        if is_standalone:
            sys.exit(app.exec())


