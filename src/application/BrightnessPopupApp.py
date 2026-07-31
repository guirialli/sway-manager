from ui.BrightnessPopup import BrightnessPopup
from application.ApplicationFactory import ApplicationFactory


class BrightnessPopupApp:
    def __init__(self) -> None:
        ApplicationFactory.buildWidget(lambda: BrightnessPopup())
