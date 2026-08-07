# Feature Spec: Power Profiles, Battery & Idle Inhibitor (`battery`, `idle`, `power`)

## 1. Overview & Objective
Manages system power states, battery conservation charging threshold (~80% vs 100%), screen idle inhibition (`swayidle`), and CPU/GPU power profiles (`power-profiles-daemon`). Supports both state toggle commands and Waybar JSON status output format for taskbar widgets.

---

## 2. Scope & Responsibilities

### In Scope
- Battery conservation charging limit toggle via sysfs (`/sys/bus/platform/drivers/ideapad_acpi/.../charge_control_end_threshold` or vendor sysfs path).
- Screen suspension inhibitor control via `swayidle` process management.
- Power profile switching (Performance, Balanced, Power-saver) via `power-profiles-daemon` / D-Bus.
- Waybar JSON payload formatting for bar integration.

---

## 3. Contracts & Interfaces

### CLI Commands
```bash
SwayManager battery [toggle|status]
SwayManager idle [toggle|status] [-s|-n|-r]
SwayManager power [toggle|status] [-p|-b|-s]
```

### Waybar JSON Protocol Contract
When invoked with `status`, commands return a single-line JSON payload to stdout:

```json
{
  "text": "⚡ 80%",
  "alt": "enabled",
  "tooltip": "Conservação de Bateria: Ativa (Limite ~80%)",
  "class": "battery-conservation-on"
}
```

---

## 4. Architecture & Component Flow

```mermaid
graph TD
    Waybar[Waybar / User CLI] -->|battery status| CLI[SwayManager battery status]
    CLI --> UC[ToggleBatteryConservationUseCase]
    UC --> SysfsRepo[SysfsBatteryRepository]
    SysfsRepo --> SysfsFile[/sys/.../charge_control_end_threshold]
    SysfsRepo-->>UC: Return BatteryState
    UC-->>CLI: State Data
    CLI-->>Waybar: JSON Payload to stdout
```

---

## 5. Security & System Privileges
- Udev rules (`udev/99-battery-conservation.rules`) grant standard user write permissions to battery threshold sysfs paths, avoiding `sudo` requirements at runtime.

---

## 6. Acceptance Criteria & Verification
- [ ] `SwayManager battery status` returns valid Waybar JSON.
- [ ] `SwayManager battery toggle` toggles threshold between 80 and 100.
- [ ] Unit tests in `tests/test_sysfs_battery_repository.py` pass cleanly.
