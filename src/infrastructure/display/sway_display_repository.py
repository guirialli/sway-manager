import subprocess
import re
from domain.display.entities import MonitoresSway, DisplaySwitchType
from domain.display.repositories import IDisplayRepository
from utils.exceptions import SwayException


class SwayDisplayRepository(IDisplayRepository):
    def get_monitors(self) -> MonitoresSway:
        self.validar_monitores()
        cmd = 'swaymsg -t get_outputs | grep "\\"name\\":"'
        processo = subprocess.run(cmd, shell=True, text=True, capture_output=True)
        monitores = re.findall(r'"name":\s*"([^"]+)"', processo.stdout)

        if monitores[1].startswith("eDP") or monitores[0].endswith("2"):
            return MonitoresSway(interno=monitores[1], externo=monitores[0])

        return MonitoresSway(interno=monitores[0], externo=monitores[1])

    def validar_monitores(self):
        cmd = 'swaymsg -t get_outputs | grep "\\"name\\":"'
        processo = subprocess.run(cmd, shell=True, text=True, capture_output=True)
        monitores = re.findall(r'"name":\s*"([^"]+)"', processo.stdout)
        if len(monitores) < 2:
            raise ValueError("Apenas um monitor encontrado!")

    def apply_config(self, mode: DisplaySwitchType) -> None:
        monitor = self.get_monitors()

        if DisplaySwitchType.PC_ONLY == mode:
            cmd = f"swaymsg output {monitor.interno} enable, output {monitor.externo} disable"
        elif DisplaySwitchType.MONITOR_ONLY == mode:
            cmd = f"swaymsg output {monitor.interno} disable, output {monitor.externo} enable"
        elif DisplaySwitchType.EXTEND == mode:
            cmd = f"swaymsg output {monitor.interno} enable, output {monitor.externo} enable"
        elif DisplaySwitchType.DUPLICATE == mode:
            cmd = f"swaymsg output {monitor.interno} enable, output {monitor.externo} enable; pkill wl-mirror; wl-mirror {monitor.interno}"
            subprocess.run(cmd, shell=True, check=True)
            return

        subprocess.run(cmd, shell=True, check=True)

    def recarregar_sway(self) -> None:
        subprocess.run(["swaymsg", "reload"])

    def get_connected_monitors_count(self) -> int:
        try:
            import json
            res = subprocess.run(["swaymsg", "-t", "get_outputs"], capture_output=True, text=True)
            if res.returncode == 0 and res.stdout:
                outputs = json.loads(res.stdout)
                return len(outputs)
        except Exception:
            pass
        return 1

    def get_current_layout(self) -> DisplaySwitchType:
        try:
            import json
            res = subprocess.run(["swaymsg", "-t", "get_outputs"], capture_output=True, text=True)
            if res.returncode == 0 and res.stdout:
                outputs = json.loads(res.stdout)
                monitors = self.get_monitors()
                int_active = any(o.get("name") == monitors.interno and o.get("active", True) for o in outputs)
                ext_active = any(o.get("name") == monitors.externo and o.get("active", True) for o in outputs)

                if int_active and not ext_active:
                    return DisplaySwitchType.PC_ONLY
                elif ext_active and not int_active:
                    return DisplaySwitchType.MONITOR_ONLY
                elif int_active and ext_active:
                    ps = subprocess.run(["pgrep", "wl-mirror"], capture_output=True)
                    if ps.returncode == 0:
                        return DisplaySwitchType.DUPLICATE
                    return DisplaySwitchType.EXTEND
        except Exception:
            pass
        return DisplaySwitchType.EXTEND
