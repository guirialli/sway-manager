from ui.MonitorSwap import MonitorSwap, MonitorCmd
from application.ApplicationFactory import ApplicationFactory
from service.SwayService import SwayService


class MonitorSwapApp:
    def __init__(self, arg: str | None = None) -> None:
        if arg:
            MonitorCmd(arg)
            return

        SwayService.validar_monitores()
        ApplicationFactory.buildWidget(lambda: MonitorSwap())
