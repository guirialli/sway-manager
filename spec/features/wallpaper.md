# Feature Spec: Wallpaper Manager & Picker (`wallpaper`)

## 1. Overview & Objective
Provides an interactive image gallery and wallpaper management system for Sway/SwayFX. Supports scanning wallpaper directories, background image loading, thumbnail caching, and setting wallpaper across all outputs using Sway wallpaper utilities.

---

## 2. Scope & Responsibilities

### In Scope
- Scanning designated wallpaper folders (`~/Pictures/Wallpapers/` default or custom directory).
- Generating image thumbnails asynchronously (`CarregadorDeImagens`).
- Displaying wallpapers in a PySide6 grid view (`WallpaperPickerWindow`).
- Applying selected wallpaper using `swaybg`, `swaymsg output * bg <path> fill`, or configured theme helper.
- Saving preferred wallpaper directory to JSON config.

---

## 3. Contracts & Interfaces

### CLI Command
```bash
SwayManager wallpaper [folder_path]
```

### Configuration Key (`JSONConfigRepository`)
- Key: `wallpaper_folder`
- Default Value: `~/Pictures/Wallpapers/`

---

## 4. Architecture & Component Flow

```mermaid
graph TD
    CLI[SwayManager wallpaper] --> UC[SetWallpaperUseCase]
    UC --> Repo[SwayWallpaperRepository]
    CLI --> UI[WallpaperPickerWindow]
    UI --> Loader[CarregadorDeImagens Worker Thread]
    Loader --> UI Grid[Image List Component]
    UI Grid -->|On Select| UC
    UC --> SwayBG[Subprocess: swaymsg output * bg fill]
```

---

## 5. UI / UX Specifications
- **Gallery Grid**: Fluid grid layout with image thumbnails, filename labels, and selection borders.
- **Asynchronous Loading**: Images loaded in background worker thread to prevent UI freezing.
- **Memory Management**: Clears pixmap cache (`QPixmapCache.clear()`) upon dialog close.

---

## 6. Acceptance Criteria & Verification
- [ ] `SwayManager wallpaper` opens the wallpaper picker GUI.
- [ ] Selecting an image applies it instantly as the active wallpaper.
- [ ] Unit tests in `tests/test_wallpaper_repository.py` pass cleanly.
