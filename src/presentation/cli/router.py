import sys
from presentation.cli.handlers import CLIHandlers


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
  theme [toggle|status]    Alterna o tema (Dark/Light) do GTK, Qt e Foot ou retorna o status (Waybar JSON).
  power [toggle|status] [flag] Alterna o perfil de energia (-p/-b/-s) ou retorna o status (Waybar JSON).
  screenshot [full|area|window] Tira uma captura de tela e copia para a área de transferência.
  menu [categoria]         Abre o lançador de aplicativos Wofi customizado.
  -h, --help               Exibe esta mensagem de ajuda.
"""
    print(help_text.strip())


def run_cli():
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
            CLIHandlers.handle_settings()
        elif app == "monitor":
            CLIHandlers.handle_monitor(args)
        elif app == "wallpaper":
            CLIHandlers.handle_wallpaper(args)
        elif app == "osd":
            CLIHandlers.handle_osd(args)
        elif app in ("brilho", "brightness"):
            CLIHandlers.handle_brightness(args)
        elif app == "battery":
            CLIHandlers.handle_battery(args)
        elif app == "idle":
            CLIHandlers.handle_idle(args)
        elif app == "theme":
            CLIHandlers.handle_theme(args)
        elif app == "power":
            CLIHandlers.handle_power(args)
        elif app == "screenshot":
            CLIHandlers.handle_screenshot(args)
        elif app == "menu":
            CLIHandlers.handle_menu(args)
        else:
            print(f"Comando '{app}' não reconhecido pelo SwayManager.\n")
            show_help()

    except Exception as ex:
        print(ex)
