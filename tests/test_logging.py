import os
import time
import unittest
import tempfile
from unittest.mock import patch

from infrastructure.logging.async_logger import AsyncLogger


class TestAsyncLogger(unittest.TestCase):
    def test_logger_enqueue_and_file_write(self):
        temp_dir = tempfile.mkdtemp()

        with patch("infrastructure.logging.async_logger.get_logs_directory", return_value=temp_dir):
            logger = AsyncLogger()
            logger.info("Test info message")
            logger.cmd("Test command message")
            logger.error("Test error message")

            # Aguarda a worker thread consumir a fila de logs
            time.sleep(0.3)

            log_path = logger.get_today_log_path()
            self.assertTrue(os.path.exists(log_path))

            with open(log_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("[INFO] Test info message", content)
            self.assertIn("[CMD] Test command message", content)
            self.assertIn("[ERROR] Test error message", content)

        # Cleanup
        if os.path.exists(log_path):
            os.remove(log_path)
        os.rmdir(temp_dir)


if __name__ == "__main__":
    unittest.main()
