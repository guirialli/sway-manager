import os
import subprocess
import json


class BatteryService:
    @classmethod
    def _find_control_file(cls) -> str | None:
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

        # Dynamic fallback
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

    @classmethod
    def status(cls):
        control_file = cls._find_control_file()
        if not control_file:
            print(
                json.dumps(
                    {
                        "text": "ERR",
                        "class": "error",
                        "tooltip": "Modo de conservação não suportado",
                    }
                )
            )
            return

        try:
            with open(control_file, "r") as f:
                val = int(f.read().strip())
        except Exception:
            val = 0

        if val in (1, 60, 80):
            print(
                json.dumps(
                    {
                        "text": "",
                        "class": "conservation-on",
                        "tooltip": "Conservação de Bateria: LIGADO (~80%)",
                    }
                )
            )
        else:
            print(
                json.dumps(
                    {
                        "text": "",
                        "class": "conservation-off",
                        "tooltip": "Conservação de Bateria: DESLIGADO (100%)",
                    }
                )
            )

    @classmethod
    def toggle(cls):
        control_file = cls._find_control_file()
        if not control_file:
            print("ERROR: Arquivo de controle não encontrado.")
            subprocess.run(
                [
                    "notify-send",
                    "-u",
                    "critical",
                    "Bateria",
                    "Conservação não suportada neste hardware.",
                ]
            )
            return

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

        subprocess.run(["notify-send", "-u", "normal", "Bateria", msg])
        print(msg)
