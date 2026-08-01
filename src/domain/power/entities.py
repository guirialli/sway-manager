from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class BatteryState:
    is_supported: bool
    is_conservation_on: bool
    raw_value: int
    control_file: Optional[str] = None

    def to_waybar_json(self) -> dict:
        if not self.is_supported:
            return {
                "text": "ERR",
                "class": "error",
                "tooltip": "Modo de conservação não suportado",
            }
        if self.is_conservation_on:
            return {
                "text": "",
                "class": "conservation-on",
                "tooltip": "Conservação de Bateria: LIGADO (~80%)",
            }
        return {
            "text": "",
            "class": "conservation-off",
            "tooltip": "Conservação de Bateria: DESLIGADO (100%)",
        }


@dataclass(frozen=True)
class IdleState:
    is_running: bool

    def to_waybar_json(self) -> dict:
        if not self.is_running:
            return {
                "text": "",
                "tooltip": "Suspensão desativada",
                "class": "idle-off",
            }
        return {
            "text": "",
            "tooltip": "Suspensão ativada",
            "class": "idle-on",
        }


@dataclass(frozen=True)
class PowerProfileState:
    active_profile: str

    ICONS = {"power-saver": "", "balanced": "", "performance": ""}
    PROFILE_NAMES = {
        "power-saver": "economia de energia",
        "balanced": "equilibrado",
        "performance": "desempenho máximo",
    }

    def to_waybar_json(self) -> dict:
        icon = self.ICONS.get(self.active_profile, "")
        return {"text": icon, "tooltip": f"Power Profile: {self.active_profile}"}
