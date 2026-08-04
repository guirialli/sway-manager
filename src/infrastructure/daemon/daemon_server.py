import os
import sys
import json
import io
import socket
import traceback
import threading
import contextlib
from typing import List
from PySide6.QtCore import QCoreApplication, QTimer
from PySide6.QtNetwork import QLocalServer, QLocalSocket


from infrastructure.daemon.daemon_client import get_socket_path
from infrastructure.logging.async_logger import get_logger
from infrastructure.daemon.server.gui import GuiServerHandler, GUI_COMMANDS
from infrastructure.daemon.server.cli import CliServerHandler, INTERACTIVE_COMMANDS
from infrastructure.daemon.server.config.gui_config import setup_qt_environment, setup_qt_cache
from infrastructure.daemon.server.config.cli_config import setup_exception_handlers



class SwayManagerDaemon:
    def __init__(self):
        self.server = QLocalServer()
        self.socket_path = get_socket_path()
        self.logger = get_logger()

    def is_daemon_running(self) -> bool:
        if not os.path.exists(self.socket_path):
            return False
        try:
            client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            client.settimeout(0.2)
            client.connect(self.socket_path)
            client.close()
            return True
        except Exception:
            return False

    def start(self):
        # Configura resiliência do Qt Wayland e manipuladores globais de exceção
        setup_qt_environment()
        setup_exception_handlers(self.logger)

        # Evita execução de múltiplas instâncias do daemon
        if self.is_daemon_running():
            print("🚀 SwayManager Daemon já está em execução.")
            sys.exit(0)

        # Configura a aplicação Qt persistente em modo Core (sem conexão Wayland GUI persistente)
        app = QCoreApplication.instance()
        if not app:
            app = QCoreApplication(sys.argv)

        # Restringe o cache interno de pixmaps do Qt para no máximo 2MB
        setup_qt_cache()


        # Remove socket estático anterior se existia arquivo residual
        if os.path.exists(self.socket_path):
            try:
                os.remove(self.socket_path)
            except Exception:
                pass
        QLocalServer.removeServer(self.socket_path)

        if not self.server.listen(self.socket_path):
            err_msg = f"❌ Erro ao iniciar daemon no socket: {self.socket_path}"
            print(err_msg)
            self.logger.error(err_msg)
            sys.exit(1)

        msg = f"🚀 SwayManager Daemon rodando no socket: {self.socket_path}"
        print(msg)
        self.logger.info(msg)

        # Pré-carrega cache do Wofi / DesktopParser sob demanda apenas ao iniciar
        try:
            from infrastructure.menu.wofi_launcher import WofiRepository
            wofi_repo = WofiRepository()
            items = wofi_repo.preload_cache()
            cache_msg = f"📦 Cache do Wofi e aplicativos pré-carregado em RAM ({len(items)} itens)!"
            print(cache_msg)
            self.logger.info(cache_msg)
        except Exception as ex:
            warn_msg = f"⚠️ Aviso ao pré-carregar cache: {ex}\n{traceback.format_exc().strip()}"
            print(warn_msg)
            self.logger.error(warn_msg)

        # Conecta sinal de novas conexões
        self.server.newConnection.connect(self._handle_new_connection)

        # Configura timer periódico de limpeza de memória (Garbage Collection) a cada 30 segundos
        self.gc_timer = QTimer()
        self.gc_timer.timeout.connect(self.perform_gc)
        self.gc_timer.start(30000)

        # Executa o loop principal do Qt
        sys.exit(app.exec())

    def perform_gc(self):
        """
        Executa a Coleta de Lixo do Python (gc.collect), limpa o cache interno de pixmaps do Qt
        e invoca malloc_trim(0) da libc para devolver páginas de heap desalocadas ao kernel Linux.
        """
        try:
            import gc
            import ctypes
            from PySide6.QtGui import QPixmapCache
            QPixmapCache.clear()
            gc.collect()
            try:
                ctypes.CDLL("libc.so.6").malloc_trim(0)
            except Exception:
                pass
        except Exception:
            pass

    def _handle_new_connection(self):
        try:
            socket_conn = self.server.nextPendingConnection()
            if not socket_conn:
                return

            socket_conn.readyRead.connect(lambda: self._process_request(socket_conn))
        except Exception as ex:
            self.logger.error(f"Erro ao aceitar conexão no socket: {ex}\n{traceback.format_exc().strip()}")

    def _process_request(self, socket_conn: QLocalSocket):
        try:
            try:
                socket_conn.readyRead.disconnect()
            except Exception:
                pass

            raw_bytes = socket_conn.readLine().data()
            if not raw_bytes:
                socket_conn.disconnectFromServer()
                return

            raw_data = raw_bytes.decode("utf-8").strip()
            if not raw_data:
                socket_conn.disconnectFromServer()
                return

            request = json.loads(raw_data)
            args: List[str] = request.get("args", [])
            cmd_str = " ".join(args[1:]) if len(args) > 1 else "help"
            self.logger.cmd(f"Executando comando via socket: '{cmd_str}'")

            cmd = str(args[1]).lower() if len(args) > 1 else ""

            if CliServerHandler.is_interactive(cmd):
                # Comandos gráficos/interativos: responde a linha JSON imediatamente (< 1ms)
                response = {"stdout": "", "stderr": "", "exit_code": 0}
                socket_conn.write((json.dumps(response) + "\n").encode("utf-8"))
                socket_conn.flush()
                socket_conn.disconnectFromServer()

                # Agenda a execução na thread principal do Qt
                QTimer.singleShot(0, lambda: self._safe_dispatch_command(args))
                return
            else:
                # Comandos síncronos com saída de texto (ex: status da Waybar)
                stdout_buf = io.StringIO()
                stderr_buf = io.StringIO()
                exit_code = 0

                with contextlib.redirect_stdout(stdout_buf), contextlib.redirect_stderr(stderr_buf):
                    self._safe_dispatch_command(args)

                response = {
                    "stdout": stdout_buf.getvalue(),
                    "stderr": stderr_buf.getvalue(),
                    "exit_code": exit_code
                }
                socket_conn.write((json.dumps(response) + "\n").encode("utf-8"))
                socket_conn.flush()
                socket_conn.disconnectFromServer()

        except Exception as ex:
            err_text = f"Erro no daemon ao processar requisição: {ex}\n{traceback.format_exc().strip()}"
            self.logger.error(err_text)
            response = {"stdout": "", "stderr": f"Erro no daemon: {ex}\n", "exit_code": 1}
            try:
                socket_conn.write((json.dumps(response) + "\n").encode("utf-8"))
                socket_conn.flush()
                socket_conn.disconnectFromServer()
            except Exception:
                pass

    def _spawn_standalone_gui(self, args: List[str]):
        """
        Delega ao GuiServerHandler a execução de janelas GUI em subprocessos isolados.
        """
        GuiServerHandler.handle_gui_command(args, logger=self.logger)

    def _safe_dispatch_command(self, args: List[str]):
        """
        Executa o despacho do comando com proteção de exceções de nível superior,
        garantindo que qualquer falha em um handler nunca derrube o daemon.
        """
        if not args or len(args) < 2:
            return

        cmd = str(args[1]).lower()

        if GuiServerHandler.is_gui_command(cmd):
            self._spawn_standalone_gui(args)
            return

        try:
            CliServerHandler.handle_cli_command(args, logger=self.logger)
        except Exception:
            pass
        finally:
            self.perform_gc()


