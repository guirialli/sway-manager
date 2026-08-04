import os


def setup_qt_environment():
    """
    Configura variáveis de ambiente do Qt para resiliência no Wayland.
    Executado apenas quando o ambiente Qt precisa ser inicializado.
    """
    os.environ.setdefault("QT_WAYLAND_RECONNECT", "1")
    os.environ.setdefault("QT_QPA_PLATFORM", "wayland;xcb")


def setup_qt_cache():
    """
    Restringe o cache interno de pixmaps do Qt para no máximo 2MB.
    """
    try:
        from PySide6.QtGui import QPixmapCache

        QPixmapCache.setCacheLimit(2048)
    except Exception:
        pass
