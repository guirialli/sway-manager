import json
import subprocess
from typing import Callable

from portal.models import PortalSource, PortalSourceType
from portal.exceptions import SwayNotAvailableError


class SwayOutputsProvider:
    """Discovers active outputs using the Sway IPC (swaymsg)."""

    def __init__(
        self,
        run: Callable[..., subprocess.CompletedProcess] = subprocess.run,
    ) -> None:
        self._run = run

    def get_outputs(self) -> list[PortalSource]:
        result = self._run(
            ["swaymsg", "-t", "get_outputs", "-r"],
            capture_output=True,
            text=True,
        )

        if result.returncode != 0:
            raise SwayNotAvailableError(
                f"swaymsg failed (code {result.returncode}): {result.stderr.strip()}"
            )

        if not result.stdout.strip():
            raise SwayNotAvailableError("swaymsg returned empty output")

        try:
            outputs = json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise SwayNotAvailableError(f"invalid JSON from swaymsg: {exc}") from exc

        sources: list[PortalSource] = []
        for output in outputs:
            if not output.get("active"):
                continue

            name = output.get("name")
            if not name:
                continue

            current_mode = output.get("current_mode") or {}
            width = current_mode.get("width", 0)
            height = current_mode.get("height", 0)
            refresh = current_mode.get("refresh", 0)
            scale = output.get("scale", 1.0)
            transform = output.get("transform", "normal")
            rect = output.get("rect") or {}
            pos_x = rect.get("x", 0)
            pos_y = rect.get("y", 0)

            make = (output.get("make") or "").strip()
            model = (output.get("model") or "").strip()
            serial = (output.get("serial") or "").strip()

            label_parts = [p for p in (make, model) if p]
            label = " ".join(label_parts) if label_parts else name
            if serial:
                label = f"{label} ({serial})"

            detail_lines = [name]
            if width and height:
                detail_lines.append(f"{width} × {height}")
            if refresh:
                detail_lines.append(f"{refresh / 1000:.1f} Hz")
            detail_lines.append(f"Escala: {scale}")
            if transform and transform != "normal":
                detail_lines.append(f"Rotação: {transform}")
            detail_lines.append(f"Posição: {pos_x},{pos_y}")
            if output.get("primary"):
                detail_lines.append("Principal")

            sources.append(
                PortalSource(
                    id=name,
                    source_type=PortalSourceType.MONITOR,
                    label=label,
                    details="\n".join(detail_lines),
                    is_primary=bool(output.get("primary")),
                    is_focused=bool(output.get("focused")),
                    extra={"output": output},
                )
            )

        if not sources:
            raise SwayNotAvailableError("nenhuma tela ativa encontrada")

        return sources
