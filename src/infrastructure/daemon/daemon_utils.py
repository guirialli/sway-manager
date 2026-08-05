import os
import sys
from typing import List


def get_binary_path(binary_name: str) -> List[str]:
    """
    Retorna o comando de execução para um binário do SwayManager ('SwayManager' ou 'SwayManagerGUI').
    Em modo compilado (Nuitka), busca o executável no mesmo diretório do processo atual.
    Em modo desenvolvimento (Python script), retorna [sys.executable, caminho_do_script.py].
    """
    is_compiled = getattr(sys, "frozen", False) or not sys.argv[0].endswith(".py")

    if is_compiled:
        exec_dir = os.path.dirname(os.path.realpath(sys.executable))

        for target in (binary_name, binary_name.lower(), binary_name.upper()):
            path = os.path.join(exec_dir, target)
            if os.path.exists(path):
                return [path]
        return [binary_name]
    else:
        script_dir = os.path.dirname(os.path.realpath(sys.argv[0]))
        name_lower = binary_name.lower()
        if name_lower in ("swaymanagergui", "sway-manager-gui", "gui"):
            gui_script = os.path.join(script_dir, "gui_main.py")
            if os.path.exists(gui_script):
                return [sys.executable, gui_script]
        elif name_lower in ("swaymanager", "sway-manager", "client", "daemon", "main"):
            main_script = os.path.join(script_dir, "main.py")
            if os.path.exists(main_script):
                return [sys.executable, main_script]
        return [binary_name]
