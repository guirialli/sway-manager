# Domain Spec: Power & Idle Subsystem

## 1. Overview & Responsibilities

The **Power Domain** coordinates battery charge threshold control (Lenovo IdeaPad conservation mode), system idle inhibition (`swayidle`), screen locking (`swaylock`), and CPU power profile cycling (`power-profiles-daemon`).

---

## 2. Component Mapping

| Layer | Component | File Path | Responsibility |
|---|---|---|---|
| Domain | `BatteryState`, `IdleState`, `PowerProfileState` | `src/domain/power/entities.py` | Power state entities and Waybar JSON serializers |
| Domain | `IBatteryRepository`, `IIdleRepository`, `ILockRepository`, `IPowerRepository` | `src/domain/power/repositories.py` | Abstract power repository interfaces |
| Application | `ToggleBatteryConservationUseCase`, `ToggleIdleUseCase`, `LockScreenUseCase`, `TogglePowerProfileUseCase` | `src/application/power/` | Power manage use cases |
| Infrastructure | `SysfsBatteryRepository` | `src/infrastructure/power/sysfs_battery_repository.py` | Sysfs ACPI driver wrapper |
| Infrastructure | `SwayIdleRepository` | `src/infrastructure/power/swayidle_repository.py` | `swayidle` daemon manager |
| Infrastructure | `SwayLockRepository` | `src/infrastructure/power/swaylock_repository.py` | `swaylock` execution wrapper |
| Infrastructure | `PowerProfilesRepository` | `src/infrastructure/power/powerprofiles_repository.py` | D-Bus / `powerprofilesctl` wrapper |

---

## 3. Subsystem Breakdown

### 3.1 Battery Conservation Mode (`SysfsBatteryRepository`)
- Target Sysfs File: `/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode`
- Values: `1` (Charge threshold ~80%), `0` (Full charge 100%).
- Requires permission granted by `udev/99-battery-conservation.rules`.

### 3.2 System Idle Inhibitor (`SwayIdleRepository`)
- Controls background `swayidle` process.
- Toggle command kills or spawns `swayidle` daemon.

### 3.3 Screen Locker (`SwayLockRepository`)
- Launches `swaylock -f` with wallpaper background resolution.

### 3.4 Power Profiles (`PowerProfilesRepository`)
- Cycles profiles: `power-saver` $\rightarrow$ `balanced` $\rightarrow$ `performance`.
- Queries status via `powerprofilesctl get` or D-Bus service `net.hadess.PowerProfiles`.

---

## 4. Waybar Integration Payloads

Status commands emit structured Waybar JSON objects to standard output:

```json
// battery status:
{"text": "", "class": "conservation-on", "tooltip": "Conservação de Bateria: LIGADO (~80%)"}

// idle status:
{"text": "", "tooltip": "Suspensão ativada", "class": "idle-on"}

// power status:
{"text": "", "tooltip": "Power Profile: performance"}
```
