# SwayManager — Configuration Specification

## 1. Overview & Location Hierarchy

SwayManager stores configuration settings, logs, and system rules in standard Linux user and system paths following the XDG Base Directory specification.

```text
~/.config/sway-manager/
├── config.json                 # Persistent application settings
├── favorites.json              # Saved clipboard favorite items
└── logs/                       # Daily non-blocking application log files
    └── log-YYYY-MM-DD.txt

~/.config/xdg-desktop-portal/
└── portals.conf                # Preferred portal backends

~/.config/xdg-desktop-portal-wlr/
└── config                      # Screen sharing chooser executable path

/etc/lightdm/
└── lightdm-gtk-greeter.conf    # LightDM display manager greeter configuration

/etc/udev/rules.d/
└── 99-battery-conservation.rules # Sysfs charge threshold permission rules
```

---

## 2. Main Configuration File (`config.json`)

Managed by `JsonConfigRepository` (`src/infrastructure/config/json_config_repository.py`).

### Schema & Fields
```json
{
  "wallpaper_folder": "~/Pictures/Wallpapers",
  "current_wallpaper": "~/Pictures/Wallpapers/default.png",
  "gtk_theme": "Adwaita-dark",
  "icon_theme": "Adwaita",
  "cursor_theme": "Bibata-Modern-Ice",
  "font_name": "Sans 10",
  "power_profile": "balanced",
  "battery_conservation": false,
  "idle_enabled": true,
  "brightness_percentage": 75
}
```

### Invariants & Test Mode Safety
`JsonConfigRepository` detects test execution environments (`SWAY_MANAGER_TEST_MODE=1` or `unittest`/`pytest` in `sys.modules`). During tests, it automatically redirects configuration reads/writes to a disposable temporary file in `/tmp/sway_manager_tests/test_config.json` to prevent mutating the user's real `config.json`.

---

## 3. Environment Variables

| Variable Name | Required / Optional | Default Value | Description |
|---|---|---|---|
| `XDG_RUNTIME_DIR` | Recommended | `/tmp/user-$UID` | Base directory for Unix Domain Socket `sway-manager-$UID.sock`. |
| `QT_QPA_PLATFORM` | Optional | Set automatically to `wayland;xcb` | Forces Qt to run natively on Wayland. |
| `QT_WAYLAND_DISABLE_WINDOWDECORATION` | Optional | Set automatically to `1` | Disables client-side Qt titlebar window decorations under Sway. |
| `SWAY_MANAGER_TEST_MODE` | Optional | `0` | Forces `JsonConfigRepository` into temp file mode. |

---

## 4. LightDM Greeter Configuration (`lightdm-gtk-greeter.conf`)

Managed by `LightDMRepository` (`src/infrastructure/theme/lightdm_repository.py`).

```ini
[greeter]
background = /usr/share/backgrounds/lightdm-wallpaper.jpg
theme-name = Adwaita-dark
icon-theme-name = Adwaita
font-name = Sans 11
cursor-theme-name = Bibata-Modern-Ice
cursor-theme-size = 24
indicators = ~host;~spacer;~clock;~spacer;~session;~language;~a11y;~power
clock-format = %a, %d %b %H:%M
```

---

## 5. Udev Permission Rule (`99-battery-conservation.rules`)

Located in `udev/99-battery-conservation.rules`. Grants non-root write access (`chmod 0666`) to Lenovo IdeaPad ACPI conservation mode sysfs nodes and charge threshold nodes upon hardware discovery:

```udev
ACTION=="add|change", KERNEL=="VPC2004:*", SUBSYSTEM=="platform", RUN+="/bin/sh -c 'chmod 0666 /sys/devices/platform/ideapad_acpi/conservation_mode /sys/bus/platform/drivers/ideapad_acpi/VPC2004:*/conservation_mode /sys/%p/conservation_mode 2>/dev/null || true'"
ACTION=="add|change", SUBSYSTEM=="power_supply", ATTR{charge_control_end_threshold}=="*", RUN+="/bin/sh -c 'chmod 0666 /sys/%p/charge_control_end_threshold 2>/dev/null || true'"
```
