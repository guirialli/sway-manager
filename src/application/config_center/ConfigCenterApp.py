from ui.ConfigCenterWindow import ConfigCenterWindow
from application.ApplicationFactory import ApplicationFactory


class ConfigCenterApp:
    def __init__(self) -> None:
        ApplicationFactory.buildWidget(
            lambda: ConfigCenterWindow(),
            desktop_file_name="sway.apps.config-center",
        )
