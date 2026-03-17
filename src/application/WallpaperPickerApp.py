from application.ApplicationFactory import ApplicationFactory
from ui.WallpaperPicker import WallpapaerPicker


class WallpaperPickerApp:
    def __init__(self, pasta: str | None) -> None:
        if not pasta:
            print("Pasta de wallpaper não informada!")
            return

        ApplicationFactory.buildWidget(lambda: WallpapaerPicker(pasta))
