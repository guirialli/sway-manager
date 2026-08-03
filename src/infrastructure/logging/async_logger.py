import os
import sys
import time
import queue
import asyncio
import threading
from datetime import datetime
from typing import Optional


def get_logs_directory() -> str:
    home_config = os.path.expanduser("~/.config/sway-manager/logs")
    os.makedirs(home_config, exist_ok=True)
    return home_config


class AsyncLogger:
    _instance: Optional["AsyncLogger"] = None
    _lock = threading.Lock()

    def __init__(self):
        self.log_dir = get_logs_directory()
        self._queue: queue.Queue[str] = queue.Queue()
        self._running = True

        # Inicia thread de fundo isolada com loop asyncio para escrita não-bloqueante em disco
        self._thread = threading.Thread(target=self._run_async_worker, daemon=True)
        self._thread.start()

    @classmethod
    def get_instance(cls) -> "AsyncLogger":
        with cls._lock:
            if cls._instance is None:
                cls._instance = AsyncLogger()
            return cls._instance

    def _get_current_log_path(self) -> str:
        date_str = datetime.now().strftime("%Y-%m-%d")
        filename = f"log-{date_str}.txt"
        return os.path.join(self.log_dir, filename)

    def _run_async_worker(self):
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        loop.run_until_complete(self._worker_loop())
        loop.close()

    async def _worker_loop(self):
        while self._running:
            try:
                # Retira mensagem da fila sem bloquear
                while not self._queue.empty():
                    msg = self._queue.get_nowait()
                    log_file = self._get_current_log_path()
                    await self._async_write_line(log_file, msg)
                    self._queue.task_done()
            except Exception:
                pass
            await asyncio.sleep(0.05)

    async def _async_write_line(self, filepath: str, line: str):
        # Executa a escrita síncrona em executor para não bloquear o event loop asyncio
        def _write():
            with open(filepath, "a", encoding="utf-8") as f:
                f.write(line + "\n")
                f.flush()

        loop = asyncio.get_running_loop()
        await loop.run_in_executor(None, _write)

    def _enqueue_log(self, level: str, msg: str):
        now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
        formatted = f"[{now_str}] [{level}] {msg}"
        self._queue.put(formatted)

    def info(self, msg: str):
        self._enqueue_log("INFO", msg)

    def cmd(self, msg: str):
        self._enqueue_log("CMD", msg)

    def error(self, msg: str):
        self._enqueue_log("ERROR", msg)

    def get_today_log_path(self) -> str:
        return self._get_current_log_path()


def get_logger() -> AsyncLogger:
    return AsyncLogger.get_instance()
