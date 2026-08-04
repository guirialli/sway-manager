import os
import sys
import unittest
import tempfile
import json
import socket
from unittest.mock import patch, MagicMock

from infrastructure.daemon.daemon_client import SwayManagerClient, get_socket_path
from infrastructure.daemon.daemon_server import SwayManagerDaemon, INTERACTIVE_COMMANDS


class TestDaemonClientAndServer(unittest.TestCase):
    def test_get_socket_path_format(self):
        sock_path = get_socket_path()
        self.assertTrue(sock_path.endswith(".sock"))
        self.assertIn("sway-manager", sock_path)

    def test_interactive_commands_set(self):
        self.assertIn("menu", INTERACTIVE_COMMANDS)
        self.assertIn("screenshot", INTERACTIVE_COMMANDS)
        self.assertIn("settings", INTERACTIVE_COMMANDS)

    def test_client_send_command_daemon_not_running(self):
        with patch("os.path.exists", return_value=False):
            result = SwayManagerClient.send_command(["sway-manager", "menu"])
            self.assertFalse(result)

    def test_is_daemon_running_check(self):
        daemon = SwayManagerDaemon()
        with patch("os.path.exists", return_value=False):
            self.assertFalse(daemon.is_daemon_running())

    def test_daemon_command_exception_handled_without_crash(self):
        daemon = SwayManagerDaemon()
        daemon.logger = MagicMock()

        # Simula erro de exceção durante execução do handler
        with patch("presentation.cli.handlers.CLIHandlers.handle_battery", side_effect=ValueError("Simulated Error")):
            daemon._safe_dispatch_command(["sway-manager", "battery", "toggle"])
            # O logger deve ter sido chamado com a mensagem de erro e o traceback
            daemon.logger.error.assert_called()
            call_args = daemon.logger.error.call_args[0][0]
            self.assertIn("Falha na execução do comando 'battery'", call_args)
            self.assertIn("Simulated Error", call_args)

    def test_client_send_command_success(self):
        temp_dir = tempfile.mkdtemp()
        mock_sock_path = os.path.join(temp_dir, "test_daemon.sock")

        server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        server.bind(mock_sock_path)
        server.listen(1)

        def mock_server_response():
            conn, _ = server.accept()
            data = conn.recv(1024)
            req = json.loads(data.decode("utf-8").strip())
            resp = json.dumps({"stdout": f"OK for {req['args'][1]}", "stderr": "", "exit_code": 0}) + "\n"
            conn.sendall(resp.encode("utf-8"))
            conn.close()

        import threading
        t = threading.Thread(target=mock_server_response)
        t.start()

        with patch("infrastructure.daemon.daemon_client.get_socket_path", return_value=mock_sock_path):
            with patch("sys.stdout"):
                result = SwayManagerClient.send_command(["sway-manager", "battery", "status"])
                self.assertTrue(result)

        t.join()
        server.close()
        if os.path.exists(mock_sock_path):
            os.remove(mock_sock_path)
        os.rmdir(temp_dir)

    def test_gui_commands_dispatch_spawns_standalone(self):
        daemon = SwayManagerDaemon()
        daemon._spawn_standalone_gui = MagicMock()
        daemon._safe_dispatch_command(["sway-manager", "settings"])
        daemon._spawn_standalone_gui.assert_called_once_with(["sway-manager", "settings"])

    def test_spawn_standalone_gui_invokes_subprocess(self):
        daemon = SwayManagerDaemon()
        with patch("subprocess.Popen") as mock_popen:
            daemon._spawn_standalone_gui(["sway-manager", "settings"])
            mock_popen.assert_called_once()
            cmd_run = mock_popen.call_args[0][0]
            self.assertIn("--standalone", cmd_run)
            self.assertIn("settings", cmd_run)


if __name__ == "__main__":
    unittest.main()

