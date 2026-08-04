import os
import sys
import subprocess
import traceback
from typing import List

GUI_COMMANDS = {
    "settings",
    "config",
    "config-center",
    "monitor",
    "wallpaper",
    "osd",
    "brilho",
    "brightness",
}


class GuiServerHandler:
    @staticmethod
    def is_gui_command(cmd: str) -> bool:
        return cmd in GUI_COMMANDS

    @staticmethod
    def handle_gui_command(args: List[str], logger=None):
        """
        Inicia um processo isolado para renderizar janelas GUI com seu próprio QApplication.
        Isso garante que o daemon principal permaneça limpo sem conexões Wayland persistentes.
        """
        try:
            cmd_args = list(args)
            first_arg = cmd_args[0] if cmd_args else ""

            if (
                first_arg
                and ("sway-manager" in first_arg or "SwayManager" in first_arg)
                and not first_arg.endswith(".py")
            ):
                exec_cmd = [first_arg, "--standalone"] + cmd_args[1:]
            else:
                exec_cmd = [sys.executable, sys.argv[0], "--standalone"] + cmd_args[1:]

            subprocess.Popen(exec_cmd)
        except Exception as ex:
            if logger:
                logger.error(
                    f"Erro ao disparar processo GUI isolado: {ex}\n{traceback.format_exc().strip()}"
                )
            print(f"Erro ao disparar processo GUI isolado: {ex}", file=sys.stderr)
