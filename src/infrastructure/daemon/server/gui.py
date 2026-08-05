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
        Inicia um processo isolado utilizando SwayManagerGUI para renderizar janelas GUI com seu próprio QApplication.
        Isso garante que o daemon principal permaneça limpo sem conexões Wayland persistentes.
        """
        try:
            from infrastructure.daemon.daemon_utils import get_binary_path

            cmd_args = list(args)
            gui_cmd = get_binary_path("SwayManagerGUI")
            exec_cmd = gui_cmd + cmd_args[1:]

            subprocess.Popen(exec_cmd)
        except Exception as ex:
            if logger:
                logger.error(
                    f"Erro ao disparar processo GUI isolado: {ex}\n{traceback.format_exc().strip()}"
                )
            print(f"Erro ao disparar processo GUI isolado: {ex}", file=sys.stderr)
