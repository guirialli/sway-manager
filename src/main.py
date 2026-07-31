import sys
from application.WallpaperPickerApp import WallpaperPickerApp
from application.MonitorSwapApp import MonitorSwapApp
from utils.array import ArrayUtils
from application.BrightnessOSDApp import BrightnessOSDApp
from application.VolumeOSDApp import VolumeOSDApp
from application.config_center.ConfigCenterApp import ConfigCenterApp
from service.BatteryService import BatteryService
from service.IdleService import IdleService
from service.ThemeService import ThemeService
from service.PowerProfileService import PowerProfileService
from service.ScreenshotService import ScreenshotService


def show_help():
    help_text = """
SwayManager - Suite de Gerenciamento para Sway e SwayFX

Uso:
  SwayManager <comando> [opções]

Comandos Disponíveis:
  settings, config         Abre o painel gráfico do Control Center (Configurações).
  monitor                  Abre a janela gráfica para alternar layouts de monitores.
  wallpaper [pasta]        Abre a janela gráfica para selecionar papel de parede.
  osd brilho [up|down]     Ajusta o brilho e exibe o OSD gráfico.
  osd volume [up|down|mute] Ajusta o volume e exibe o OSD gráfico.
  battery [toggle|status]  Alterna a conservação de bateria (~80% vs 100%) ou retorna o status (Waybar JSON).
  idle [toggle|status] [flag] Alterna o inibidor de suspensão swayidle (-s/-n/-r) ou retorna o status (Waybar JSON).
  theme [toggle|status]    Alterna o tema (Dark/Light) do GTK e Foot ou retorna o status (Waybar JSON).
  power [toggle|status] [flag] Alterna o perfil de energia (-p/-b/-s) ou retorna o status (Waybar JSON).
  screenshot [full|area|window] Tira uma captura de tela e copia para a área de transferência.
  -h, --help               Exibe esta mensagem de ajuda.
"""
    print(help_text.strip())


def main():
    args = sys.argv

    if len(args) == 1:
        show_help()
        return

    app = str(args[1]).lower()

    if app in ("-h", "--help", "help"):
        show_help()
        return

    try:
        if app in ("settings", "config", "config-center"):
            ConfigCenterApp()
        elif app == "monitor":
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
        elif app in ("brilho", "brightness"):
            BrightnessOSDApp(acao=ArrayUtils.getSafe(args, 2))
        elif app == "battery":
            action = ArrayUtils.getSafe(args, 2, "toggle").lower()
            if action == "status":
                BatteryService.status()
            else:
                BatteryService.toggle()
        elif app == "idle":
            action = ArrayUtils.getSafe(args, 2, "toggle").lower()
            flag = ArrayUtils.getSafe(args, 3)
            if action == "status":
                IdleService.status()
            else:
                IdleService.toggle(flag)
        elif app == "theme":
            action = ArrayUtils.getSafe(args, 2, "toggle").lower()
            if action == "status":
                ThemeService.status()
            else:
                ThemeService.toggle()
        elif app == "power":
            action = ArrayUtils.getSafe(args, 2, "toggle").lower()
            flag = ArrayUtils.getSafe(args, 3)
            if action == "status":
                PowerProfileService.status()
            else:
                PowerProfileService.toggle(flag)
        elif app == "screenshot":
            mode = ArrayUtils.getSafe(args, 2, "full").lower()
            ScreenshotService.take(mode)
        else:
            print(f"Comando '{app}' não reconhecido pelo SwayManager.\n")
            show_help()

    except Exception as ex:
        print(ex)


if __name__ == "__main__":
    main()
