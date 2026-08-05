import sys
from presentation.cli.handlers import CLIHandlers


def run_gui_app():
    args = list(sys.argv)
    if len(args) < 2:
        print("SwayManagerGUI - Módulo de Interface Gráfica (Qt)")
        print("Uso: SwayManagerGUI <settings|monitor|wallpaper|osd> [opções]")
        sys.exit(1)

    cmd = str(args[1]).lower()
    if cmd in ("settings", "config", "config-center"):
        CLIHandlers.handle_settings()
    elif cmd == "monitor":
        CLIHandlers.handle_monitor(args)
    elif cmd == "wallpaper":
        CLIHandlers.handle_wallpaper(args)
    elif cmd in ("osd", "brilho", "brightness"):
        CLIHandlers.handle_osd(args)
    else:
        print(
            f"Comando GUI '{cmd}' não reconhecido pelo SwayManagerGUI.",
            file=sys.stderr,
        )
        sys.exit(1)


if __name__ == "__main__":
    run_gui_app()
