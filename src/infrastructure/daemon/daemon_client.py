import os
import sys
import json
import socket
from typing import List


def get_socket_path() -> str:
    runtime_dir = os.environ.get("XDG_RUNTIME_DIR")
    if not runtime_dir or not os.path.exists(runtime_dir):
        runtime_dir = f"/tmp/user-{os.getuid()}"
        os.makedirs(runtime_dir, exist_ok=True)
    return os.path.join(runtime_dir, f"sway-manager-{os.getuid()}.sock")


class SwayManagerClient:
    @staticmethod
    def send_command(args: List[str], timeout: float = 2.0) -> bool:
        """
        Tenta enviar o comando para o daemon via Unix Domain Socket.
        Retorna True se o daemon respondeu (e imprime a saída), ou False se o daemon não estiver rodando.
        """
        sock_path = get_socket_path()
        if not os.path.exists(sock_path):
            return False

        try:
            client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            client.settimeout(timeout)
            client.connect(sock_path)

            payload = json.dumps({"args": args}) + "\n"
            client.sendall(payload.encode("utf-8"))

            # Lê uma única linha JSON terminada em '\n' e fecha a conexão pelo cliente em < 1ms
            sock_file = client.makefile("r", encoding="utf-8")
            line = sock_file.readline()
            client.close()

            if not line:
                return False

            data = json.loads(line.strip())

            stdout = data.get("stdout", "")
            stderr = data.get("stderr", "")

            if stdout:
                print(stdout, end="" if stdout.endswith("\n") else "\n")
            if stderr:
                print(stderr, file=sys.stderr, end="" if stderr.endswith("\n") else "\n")

            return True
        except Exception:
            return False
