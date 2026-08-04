import gc
import sys
import ctypes
from typing import Callable
from infrastructure.daemon.server.config.gui_config import setup_qt_environment

# Ensure Qt Wayland resilience flags are set before Qt initialization
setup_qt_environment()


from PySide6.QtCore import Qt
from PySide6.QtGui import QPixmapCache
from PySide6.QtWidgets import QWidget, QApplication




def _cleanup_memory(widget: QWidget, active_list: list[QWidget]):
    if widget in active_list:
        try:
            active_list.remove(widget)
        except Exception:
            pass
    try:
        QPixmapCache.clear()
        gc.collect()
        try:
            ctypes.CDLL("libc.so.6").malloc_trim(0)
        except Exception:
            pass
    except Exception:
        pass


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
            widget.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose, True)
            cls._active_widgets.append(widget)

            widget.destroyed.connect(
                lambda: _cleanup_memory(widget, cls._active_widgets)
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
