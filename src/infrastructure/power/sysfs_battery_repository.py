import os
import subprocess
from typing import Optional
from domain.power.entities import BatteryState
from domain.power.repositories import IBatteryRepository
from domain.notification.entities import Notification
from domain.notification.repositories import INotificationRepository
from infrastructure.notification.desktop_notification_repository import (
    DesktopNotificationRepository,
)


class SysfsBatteryRepository(IBatteryRepository):
    def __init__(self, notification_repo: Optional[INotificationRepository] = None):
        self.notification_repo = notification_repo or DesktopNotificationRepository()

    def _find_control_file(self) -> Optional[str]:
        paths = [
            "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode",
            "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:01/conservation_mode",
            "/sys/devices/platform/ideapad_acpi/conservation_mode",
            "/sys/class/power_supply/BAT0/charge_control_end_threshold",
            "/sys/class/power_supply/BAT1/charge_control_end_threshold",
        ]

        for p in paths:
            if os.path.isfile(p):
                return p

        try:
            res = subprocess.run(
                "find /sys/devices -name conservation_mode 2>/dev/null | head -n 1",
                shell=True,
                text=True,
                capture_output=True,
            )
            found = res.stdout.strip()
            if found and os.path.isfile(found):
                return found
        except Exception:
            pass

        return None

    def get_state(self) -> BatteryState:
        control_file = self._find_control_file()
        if not control_file:
            return BatteryState(is_supported=False, is_conservation_on=False, raw_value=0)

        try:
            with open(control_file, "r") as f:
                val = int(f.read().strip())
        except Exception:
            val = 0

        is_on = val in (1, 60, 80)
        return BatteryState(
            is_supported=True,
            is_conservation_on=is_on,
            raw_value=val,
            control_file=control_file,
        )

    def toggle(self) -> tuple[bool, str]:
        control_file = self._find_control_file()
        if not control_file:
            msg = "Conservação não suportada neste hardware."
            self.notification_repo.notify(
                Notification(title="Bateria", message=msg, urgency="critical")
            )
            return False, "ERROR: Arquivo de controle não encontrado."

        try:
            with open(control_file, "r") as f:
                val = int(f.read().strip())
        except Exception:
            val = 0

        if val == 1:
            new_val = 0
            msg = "Conservação Desligada (Carregando até 100%)"
        elif val == 0:
            new_val = 1
            msg = "Conservação Ligada (Limitado a 80%)"
        elif val in (60, 80):
            new_val = 100
            msg = "Conservação Desligada (Carregando até 100%)"
        else:
            new_val = 80
            msg = "Conservação Ligada (Limitado a 80%)"

        if os.access(control_file, os.W_OK):
            with open(control_file, "w") as f:
                f.write(str(new_val))
        else:
            cmd = f"echo {new_val} > '{control_file}'"
            if subprocess.run("command -v pkexec", shell=True).returncode == 0:
                subprocess.run(["pkexec", "sh", "-c", cmd])
            else:
                subprocess.run(["sudo", "sh", "-c", cmd])

        self.notification_repo.notify(
            Notification(title="Bateria", message=msg, urgency="normal")
        )
        return True, msg
