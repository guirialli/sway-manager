# SwayManager

**SwayManager** is a Rust desktop suite and CLI management tool developed for **Sway** and **SwayFX** Wayland compositors. The persistent daemon (`SwayManager`) and the GUI (`SwayManagerGUI`, Tauri v2 + WebKitGTK) provide display layouts, wallpaper pickers, volume/brightness OSDs, battery conservation mode, idle lock inhibitors, theme switching, power profiles, and screenshots.

---

## 🌟 Key Features

### 🖥️ Graphical Windows (Tauri v2)
- **Control Center (`SwayManager settings`)**: Configurator panel with tabs for wallpapers, appearance (GTK & Qt dark/light), power/battery modes, idle inhibitors, and **LightDM greeter settings** (login wallpaper, GTK/icon themes, fonts, clock format).
- **Monitor Display Switcher (`SwayManager monitor`)**: Multi-monitor configuration dialog to arrange, mirror, or disable displays.
- **Wallpaper Picker (`SwayManager wallpaper [folder]`)**: Visual grid picker dialog to preview and apply wallpapers via `swaybg`.
- **Volume OSD (`SwayManager osd volume [up|down|mute]`)**: Volume OSD overlay (singleton: rapid presses update in place).
- **Brightness OSD (`SwayManager osd brilho [up|down]`)**: Screen brightness OSD overlay.

### ⚙️ CLI Services & Waybar Integrations
- **Battery Conservation (`SwayManager battery [toggle|status]`)**: Toggles battery charge thresholds (~80% limit vs 100% full charge) across Lenovo IdeaPad and generic Linux sysfs attributes.
- **Idle Inhibitor (`SwayManager idle [toggle|status]`)**: Manages `swayidle` sleep and lock daemon states.
- **Theme Switcher (`SwayManager theme [toggle|status]`)**: Toggles GTK color schemes (`Adwaita` / `Adwaita-dark`), Qt themes (`qt5ct`, `qt6ct`, `Kvantum`, `KDE Globals`), and Foot terminal color themes (`white.ini` / `black.ini`).
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
| `SwayManager theme` | `toggle` \| `status` | Toggles GTK, Qt & Foot themes or outputs Waybar JSON status. |
| `SwayManager power` | `toggle [flag]` \| `status` | Cycles `powerprofilesctl` profile or outputs Waybar JSON status. |
| `SwayManager screenshot` | `full` \| `area` \| `window` | Captures screenshot and copies image to clipboard. |

---

## 🛠️ Installation & Setup

SwayManager is integrated into the dotfiles installer `install.sh`:

```bash
# Install SwayManager binaries at ~/.config/sway/bin/
./install.sh --no-udev
```

### Manual Build
1. Ensure Rust (cargo) and Tauri Linux deps are installed:
   ```bash
   sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
   ```
2. Build daemon + GUI (runs all test suites first):
   ```bash
   ./build.sh && ./out/main.dist/SwayManager --help
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
