import os
import json
from typing import Any, Optional


class JsonConfigRepository:
    def __init__(self, config_path: Optional[str] = None):
        self.config_path = os.path.expanduser(
            config_path or "~/.config/sway-manager/config.json"
        )

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
