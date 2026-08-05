import sys
import traceback
from typing import List

INTERACTIVE_COMMANDS = {
    "menu",
    "screenshot",
    "clipboard",
    "clip",
    "settings",
    "config",
    "config-center",
    "monitor",
    "wallpaper",
    "osd",
    "brilho",
    "brightness",
    "lock",
}


class CliServerHandler:
    @staticmethod
    def is_interactive(cmd: str) -> bool:
        return cmd in INTERACTIVE_COMMANDS

    @staticmethod
    def handle_cli_command(args: List[str], logger=None):
        if not args or len(args) < 2:
            return

        cmd = str(args[1]).lower()

        from presentation.cli.handlers import CLIHandlers

        try:
            if cmd == "battery":
                CLIHandlers.handle_battery(args)
            elif cmd == "idle":
                CLIHandlers.handle_idle(args)
            elif cmd == "theme":
                CLIHandlers.handle_theme(args)
            elif cmd == "power":
                CLIHandlers.handle_power(args)
            elif cmd == "screenshot":
                CLIHandlers.handle_screenshot(args)
            elif cmd == "menu":
                CLIHandlers.handle_menu(args)
            elif cmd in ("clipboard", "clip"):
                CLIHandlers.handle_clipboard(args)
            elif cmd == "lock":
                CLIHandlers.handle_lock(args)
            elif cmd in ("reload-cache", "refresh-cache"):
                from infrastructure.menu.wofi_launcher import WofiRepository

                items = WofiRepository().preload_cache()
                print(f"Cache do Wofi recarregado com sucesso ({len(items)} itens).")
                if logger:
                    logger.info(f"Cache do Wofi recarregado ({len(items)} itens).")
            else:
                print(f"Comando '{cmd}' não reconhecido pelo SwayManager.\n")
        except Exception as ex:
            tb_str = traceback.format_exc().strip()
            if logger:
                logger.error(f"Falha na execução do comando '{cmd}': {ex}\n{tb_str}")
            print(f"Erro ao executar '{cmd}': {ex}", file=sys.stderr)
            raise
