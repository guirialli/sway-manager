import os
import sys
import json
import tempfile
from typing import Any, Optional


def is_running_under_test() -> bool:
    """
    Retorna True se o código estiver rodando em ambiente de testes unitários
    (unittest, pytest, ou flag SWAY_MANAGER_TEST_MODE).
    """
    if os.environ.get("SWAY_MANAGER_TEST_MODE") == "1":
        return True
    if "unittest" in sys.modules or "pytest" in sys.modules:
        return True
    if any("unittest" in str(arg) or "pytest" in str(arg) for arg in sys.argv):
        return True
    return False


class JsonConfigRepository:
    def __init__(self, config_path: Optional[str] = None):
        if config_path:
            self.config_path = os.path.expanduser(config_path)
        elif is_running_under_test():
            # Em ambiente de teste, utiliza arquivo temporário descartável no /tmp
            test_temp_dir = os.path.join(tempfile.gettempdir(), "sway_manager_tests")
            os.makedirs(test_temp_dir, exist_ok=True)
            self.config_path = os.path.join(test_temp_dir, "test_config.json")
        else:
            self.config_path = os.path.expanduser("~/.config/sway-manager/config.json")

    def load_config(self) -> dict[str, Any]:
        if os.path.isfile(self.config_path):
            try:
                with open(self.config_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    if isinstance(data, dict):
                        return data
            except Exception:
                pass
        return {}

    def save_config(self, data: dict[str, Any]) -> bool:
        try:
            os.makedirs(os.path.dirname(self.config_path), exist_ok=True)
            with open(self.config_path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
                f.write("\n")
            return True
        except Exception as e:
            print(f"Error saving config to {self.config_path}: {e}")
            return False

    def get_setting(self, key: str, default: Any = None) -> Any:
        config = self.load_config()
        return config.get(key, default)

    def set_setting(self, key: str, value: Any) -> bool:
        config = self.load_config()
        config[key] = value
        return self.save_config(config)
