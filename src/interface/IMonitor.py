from dataclasses import dataclass
from PySide6.QtWidgets import QWidget
from enum import Enum


@dataclass
class MonitoresSway:
    interno: str
    externo: str


class DisplaySwitchType(Enum):
    PC_ONLY = 0
    MONITOR_ONLY = 1
    EXTEND = 2
    DUPLICATE = 3


class ISwitcherUI(QWidget):
    def __init__(self) -> None:
        super().__init__()

    def confirmar_ou_reverter(self):
        raise NotImplementedError("Precisar ser impletado pela classe filha")
