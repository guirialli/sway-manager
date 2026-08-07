import configparser
import os
import shutil
import subprocess
from datetime import datetime
from pathlib import Path


class PortalConfigInstaller:
    """Installs or updates portal configuration files for SwayManager."""

    PORTALS_CONF_SECTION = "preferred"
    SCREENCAST_SECTION = "screencast"

    def __init__(self, executable_path: str | None = None) -> None:
        self.home = Path.home()
        self.bin_dir = self.home / ".config" / "sway" / "bin"
        self.executable = Path(executable_path) if executable_path else self.bin_dir / "SwayManager"

    def install(self) -> list[str]:
        """Install all managed portal settings and return a log of actions."""
        log: list[str] = []
        self._ensure_executable_exists(log)
        self._write_portals_conf(log)
        self._write_wlr_config(log)
        self._ensure_sway_session_env(log)
        self._restart_portal_services(log)
        return log

    def _ensure_executable_exists(self, log: list[str]) -> None:
        if not self.executable.exists():
            raise FileNotFoundError(f"Executável não encontrado: {self.executable}")
        log.append(f"Executável confirmado: {self.executable}")

    def _write_portals_conf(self, log: list[str]) -> None:
        portal_dir = self.home / ".config" / "xdg-desktop-portal"
        portal_dir.mkdir(parents=True, exist_ok=True)

        for filename in ("portals.conf", "sway-portals.conf"):
            path = portal_dir / filename
            self._backup_if_exists(path, log)

            config = configparser.ConfigParser()
            config.optionxform = str  # preserve case
            if path.exists():
                config.read(path)
            if not config.has_section(self.PORTALS_CONF_SECTION):
                config.add_section(self.PORTALS_CONF_SECTION)

            config.set(self.PORTALS_CONF_SECTION, "default", "gtk")
            config.set(
                self.PORTALS_CONF_SECTION,
                "org.freedesktop.impl.portal.Screenshot",
                "wlr",
            )
            config.set(
                self.PORTALS_CONF_SECTION,
                "org.freedesktop.impl.portal.ScreenCast",
                "wlr",
            )

            self._write_ini(path, config)
            log.append(f"Configuração atualizada: {path}")

    def _write_wlr_config(self, log: list[str]) -> None:
        path = self.home / ".config" / "xdg-desktop-portal-wlr" / "config"
        path.parent.mkdir(parents=True, exist_ok=True)
        self._backup_if_exists(path, log)

        config = configparser.ConfigParser()
        config.optionxform = str
        if path.exists():
            config.read(path)
        if not config.has_section(self.SCREENCAST_SECTION):
            config.add_section(self.SCREENCAST_SECTION)

        chooser_cmd = str(self.executable.resolve()) + " portal"
        config.set(self.SCREENCAST_SECTION, "chooser_cmd", chooser_cmd)
        config.set(self.SCREENCAST_SECTION, "chooser_type", "dmenu")
        if not config.has_option(self.SCREENCAST_SECTION, "max_fps"):
            config.set(self.SCREENCAST_SECTION, "max_fps", "30")

        self._write_ini(path, config)
        log.append(f"Configuração atualizada: {path}")

    def _ensure_sway_session_env(self, log: list[str]) -> None:
        sway_config = self.home / ".config" / "sway" / "config"
        if not sway_config.exists():
            log.append(f"Arquivo de configuração do Sway não encontrado: {sway_config}")
            return

        marker = "dbus-update-activation-environment"
        content = sway_config.read_text(encoding="utf-8")
        if marker in content:
            log.append(f"Variáveis de sessão já configuradas em: {sway_config}")
            return

        self._backup_if_exists(sway_config, log)
        line = (
            "exec dbus-update-activation-environment --systemd "
            "WAYLAND_DISPLAY XDG_CURRENT_DESKTOP=sway\n"
        )
        with sway_config.open("a", encoding="utf-8") as f:
            f.write("\n# SwayManager: export session variables for D-Bus services\n")
            f.write(line)
        log.append(f"Variáveis de sessão adicionadas em: {sway_config}")

    def _restart_portal_services(self, log: list[str]) -> None:
        for service in ("xdg-desktop-portal-wlr", "xdg-desktop-portal"):
            try:
                result = subprocess.run(
                    ["systemctl", "--user", "restart", service],
                    capture_output=True,
                    text=True,
                )
                if result.returncode == 0:
                    log.append(f"Serviço reiniciado: {service}")
                else:
                    log.append(
                        f"Não foi possível reiniciar {service}: {result.stderr.strip()}"
                    )
            except FileNotFoundError:
                log.append(f"systemctl não encontrado; {service} não reiniciado")

    def _backup_if_exists(self, path: Path, log: list[str]) -> None:
        if not path.exists():
            return
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        backup = Path(f"{path}.{timestamp}.bak")
        shutil.copy2(path, backup)
        log.append(f"Backup criado: {backup}")

    @staticmethod
    def _write_ini(path: Path, config: configparser.ConfigParser) -> None:
        with path.open("w", encoding="utf-8") as f:
            config.write(f)
