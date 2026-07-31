# SwayManager

**SwayManager** is a PySide6 desktop suite and CLI management tool developed for **Sway** and **SwayFX** Wayland compositors. It provides both GTK/Qt popups and CLI services for display layouts, wallpaper pickers, volume/brightness OSDs, battery conservation mode, idle lock inhibitors, theme switching, power profiles, and screenshots.

---

## 🌟 Key Features

### 🖥️ Graphical Popups (Qt / PySide6)
- **Monitor Display Switcher (`SwayManager monitor`)**: Multi-monitor configuration dialog to arrange, mirror, or disable displays.
- **Wallpaper Picker (`SwayManager wallpaper [folder]`)**: Visual grid picker dialog to preview and apply wallpapers via `swaybg`.
- **Volume OSD (`SwayManager osd volume [up|down|mute]`)**: Animated volume OSD overlay.
- **Brightness OSD (`SwayManager osd brilho [up|down]`)**: Animated screen brightness OSD overlay.

### ⚙️ CLI Services & Waybar Integrations
- **Battery Conservation (`SwayManager battery [toggle|status]`)**: Toggles battery charge thresholds (~80% limit vs 100% full charge) across Lenovo IdeaPad and generic Linux sysfs attributes.
- **Idle Inhibitor (`SwayManager idle [toggle|status]`)**: Manages `swayidle` sleep and lock daemon states.
- **Theme Switcher (`SwayManager theme [toggle|status]`)**: Toggles GTK color schemes (`Adwaita` / `Adwaita-dark`) and Foot terminal color themes (`white.ini` / `black.ini`).
- **Power Profiles (`SwayManager power [toggle|status]`)**: Cycles `powerprofilesctl` profiles (`power-saver`, `balanced`, `performance`).
- **Screenshot Utility (`SwayManager screenshot [full|area|window]`)**: Captures full-screen, region, or focused window screenshots using `grim` + `slurp` and copies image data directly to clipboard.

---

## 🚀 CLI Command Reference

| Command | Action | Description |
| :--- | :--- | :--- |
| `SwayManager settings` | `--` | Opens the Control Center (Configuracoes) GUI panel. |
| `SwayManager monitor` | `--` | Opens multi-monitor setup GUI dialog. |
| `SwayManager wallpaper` | `[folder]` | Opens wallpaper picker GUI grid dialog for given folder path. |
| `SwayManager osd volume` | `up` \| `down` \| `mute` | Adjusts volume and displays OSD overlay popup. |
| `SwayManager osd brilho` | `up` \| `down` | Adjusts display brightness and displays OSD overlay popup. |
| `SwayManager battery` | `toggle` \| `status` | Toggles battery conservation mode or outputs Waybar JSON status. |
| `SwayManager idle` | `toggle [flag]` \| `status` | Toggles `swayidle` daemon or outputs Waybar JSON status. |
| `SwayManager theme` | `toggle` \| `status` | Toggles GTK & Foot themes or outputs Waybar JSON status. |
| `SwayManager power` | `toggle [flag]` \| `status` | Cycles `powerprofilesctl` profile or outputs Waybar JSON status. |
| `SwayManager screenshot` | `full` \| `area` \| `window` | Captures screenshot and copies image to clipboard. |

---

## 🛠️ Installation & Setup

SwayManager is integrated into the dotfiles installer `install.sh`:

```bash
# Install SwayManager in a isolated virtual environment at ~/.config/sway/bin/
./install.sh --sway-manager
```

### Manual Installation
1. Ensure Python 3.10+ is installed.
2. Install Python dependencies:
   ```bash
   pip install -r sway-manager/requirements.txt
   ```
3. Run SwayManager main entry point:
   ```bash
   python3 sway-manager/src/main.py --help
   ```

---

## 🔌 Waybar Integration Example

Sample snippet from `~/.config/waybar/config.jsonc`:

```jsonc
{
  "custom/settings": {
    "format": "⚙️",
    "tooltip-format": "SwayManager Control Center",
    "on-click": "~/.config/sway/bin/SwayManager settings"
  },
  "custom/idle": {
    "interval": 2,
    "return-type": "json",
    "exec": "~/.config/sway/bin/SwayManager idle status",
    "on-click": "~/.config/sway/bin/SwayManager idle toggle"
  },
  "custom/conservation": {
    "interval": 2,
    "return-type": "json",
    "exec": "~/.config/sway/bin/SwayManager battery status",
    "on-click": "~/.config/sway/bin/SwayManager battery toggle"
  },
  "custom/theme": {
    "interval": 2,
    "return-type": "json",
    "exec": "~/.config/sway/bin/SwayManager theme status",
    "on-click": "~/.config/sway/bin/SwayManager theme toggle"
  },
  "custom/power-tlp": {
    "interval": 5,
    "return-type": "json",
    "exec": "~/.config/sway/bin/SwayManager power status",
    "on-click": "~/.config/sway/bin/SwayManager power toggle"
  }
}
```

### Keybindings in Sway (`~/.config/sway/config.d/02-bindings`)
- `$mod+comma` or `$mod+SHIFT+o`: Open SwayManager Control Center (`$sway-manager settings`).
- `$mod+Ctrl+s`: Toggle Screen Idle / Suspension (`$sway-manager idle toggle`).
- `$mod+Ctrl+h`: System Hibernation (`systemctl hibernate`).

---

## 📄 License
This project is licensed under the GPL-3.0 License.
