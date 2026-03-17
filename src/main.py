import sys
from application.WallpaperPickerApp import WallpaperPickerApp
from application.MonitorSwapApp import MonitorSwapApp
from utils.array import ArrayUtils
from application.BrightnessOSDApp import BrightnessOSDApp
from application.VolumeOSDApp import VolumeOSDApp


def main():
    args = sys.argv

    if len(args) == 1:
        print("Nenhum argumento informado!")
        return

    app = str(args[1]).lower()

    try:
        if app == "monitor":
            MonitorSwapApp(arg=ArrayUtils.getSafe(args, 2))
        elif app == "wallpaper":
            WallpaperPickerApp(pasta=ArrayUtils.getSafe(args, 2))
        elif app == "osd":
            osd = ArrayUtils.getSafe(args, 2)
            if not osd:
                print("OSD não informado!")
                return

            if osd == "brilho":
                BrightnessOSDApp(acao=ArrayUtils.getSafe(args, 3))
            elif osd == "volume":
                VolumeOSDApp(acao=ArrayUtils.getSafe(args, 3))

    except Exception as ex:
        print(ex)


if __name__ == "__main__":
    main()
