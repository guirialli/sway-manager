# Domain Spec: Theme & LightDM Subsystem

## 1. Overview & Responsibilities

The **Theme Domain** manages GTK and Qt dark/light mode appearance synchronization, Sway background wallpaper configuration, and LightDM GTK greeter settings modification.

---

## 2. Component Mapping

| Layer | Component | File Path | Responsibility |
|---|---|---|---|
| Domain | `AppearanceSettings`, `ThemeState`, `LightDMSettings` | `src/domain/theme/entities.py` | Theme data structures and options |
| Domain | `IThemeRepository`, `IWallpaperRepository`, `ILightDMRepository` | `src/domain/theme/repositories.py` | Abstract theme repository interfaces |
| Application | `ToggleThemeUseCase`, `SetWallpaperUseCase`, `UpdateLightDMUseCase` | `src/application/theme/` | Theme use cases |
| Infrastructure | `GtkQtThemeRepository` | `src/infrastructure/theme/gtk_qt_theme_repository.py` | Modifies GTK 3.0 `settings.ini` & `qt5ct.conf` |
| Infrastructure | `SwayWallpaperRepository` | `src/infrastructure/theme/sway_wallpaper_repository.py` | Sets wallpaper via Sway IPC & updates config |
| Infrastructure | `LightDMRepository` | `src/infrastructure/theme/lightdm_repository.py` | Reads/writes `/etc/lightdm/lightdm-gtk-greeter.conf` |
| Presentation | `WallpaperPickerWindow` | `src/presentation/gui/windows/wallpaper_picker.py` | Visual grid image selector |

---

## 3. Theme Synchronization Architecture

When `ToggleThemeUseCase` is executed:
1. `GtkQtThemeRepository` reads current GTK theme setting in `~/.config/gtk-3.0/settings.ini`.
2. If `Adwaita-dark`, switches to `Adwaita` (Light Mode). If `Adwaita`, switches to `Adwaita-dark` (Dark Mode).
3. Updates `gtk-theme-name` in `~/.config/gtk-3.0/settings.ini`.
4. Updates `style=Adwaita-dark` in `~/.config/qt5ct/qt5ct.conf`.
5. Sends notification: `"Tema alterado para Modo Escuro / Claro"`.

---

## 4. LightDM Greeter Configuration File (`lightdm-gtk-greeter.conf`)

`LightDMRepository` parses and modifies `/etc/lightdm/lightdm-gtk-greeter.conf` INI sections (`[greeter]`):

```ini
[greeter]
background = /usr/share/backgrounds/lightdm-wallpaper.jpg
theme-name = Adwaita-dark
icon-theme-name = Adwaita
font-name = Sans 11
cursor-theme-name = Bibata-Modern-Ice
cursor-theme-size = 24
```
