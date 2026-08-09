# SwayManager — Cyberpunk Visual Design System Specification

## 1. Vision & Aesthetic Overview

The **SwayManager Cyberpunk Design System** provides a sleek, high-tech visual identity based on deep obsidian black backgrounds (`#0D0B14`) contrasted with electric cyberpunk neon purple accents (`#9D4EDD`, `#C77DFF`, `#7B2CBF`) and subtle violet borders (`rgba(157, 78, 221, 0.25)`).

This design system was inspired by the screen sharing portal chooser dialog and is applied across all PySide6 graphical components, including windows, dialogs, OSDHUD overlays, and cards.

```mermaid
flowchart TD
    subgraph Color Palette
        VoidBlack["Window Void Black\n#0D0B14"]
        PanelBlack["Panel / Sidebar Black\n#13111C"]
        CardSurface["Card Surface\n#191624"]
        ElectricPurple["Electric Purple Accent\n#9D4EDD"]
        NeonLavender["Neon Hover Lavender\n#C77DFF"]
        VioletBorder["Glow Border\nrgba(157, 78, 221, 0.25)"]
    end

    subgraph UI Components
        ConfigCenter["ConfigCenterWindow"]
        MonitorSwap["MonitorSwapWindow"]
        WallpaperPicker["WallpaperPickerWindow"]
        PortalDialog["PortalDialog (Screencast Chooser)"]
        OSDOverlays["VolumeOSD & BrightnessOSD"]
    end

    VoidBlack --> UI Components
    CardSurface --> UI Components
    ElectricPurple --> UI Components
    VioletBorder --> UI Components
```

---

## 2. Color Palette Token Specification

### Dark Mode (Cyberpunk Black & Neon Purple)
```python
DARK_COLORS = {
    "window_bg": "#0D0B14",                 # Deep cyberpunk obsidian black
    "sidebar_bg": "#13111C",                # Dark purple-black panel
    "card_bg": "#191624",                   # Subtle purple-black card surface
    "card_border": "rgba(157, 78, 221, 0.25)", # Cyberpunk violet glowing outline
    "text_primary": "#F8F7FF",              # Crisp bright neon white
    "text_secondary": "#B8B3D0",            # Soft lavender silver
    "text_muted": "#7A7596",                # Muted violet grey
    "accent": "#9D4EDD",                    # Electric Cyberpunk Purple
    "accent_hover": "#C77DFF",              # Bright Neon Lavender Violet
    "accent_text": "#FFFFFF",
    "input_bg": "#14121F",
    "input_border": "rgba(157, 78, 221, 0.35)",
    "button_bg": "rgba(157, 78, 221, 0.12)",
    "button_hover": "rgba(157, 78, 221, 0.25)",
    "button_border": "rgba(199, 125, 255, 0.3)",
    "selection_bg": "#7B2CBF",
    "scrollbar_handle": "rgba(157, 78, 221, 0.4)",
    "scrollbar_handle_hover": "rgba(199, 125, 255, 0.7)",
    "osd_bg": "rgba(13, 11, 20, 235)",      # Cyberpunk translucent glass OSD
    "osd_border": "rgba(199, 125, 255, 0.4)", # Glowing neon border
    "progress_bg": "rgba(157, 78, 221, 0.2)",
    "progress_chunk": "#C77DFF",            # Neon Purple chunk
}
```

---

## 3. UI Component Styling Guidelines

### 3.1 Card Components (`SourceCard`, `QFrame.appleCard`)
- **Background**: `#191624`
- **Border**: `1px solid rgba(157, 78, 221, 0.25)`
- **Selected State**: Background `#7B2CBF` with bright glowing border `2px solid #C77DFF`.

### 3.2 Buttons (`QPushButton`)
- **Secondary Buttons**: Translucent purple background `rgba(157, 78, 221, 0.12)` with border `rgba(199, 125, 255, 0.3)`. Hover brightens background to `rgba(157, 78, 221, 0.25)`.
- **Primary Buttons (`.primaryButton`)**: Solid Electric Purple `#9D4EDD` with hover `#C77DFF`.

### 3.3 On-Screen Displays (`VolumeOSD`, `BrightnessOSD`)
- **Background**: High-density translucent cyberpunk dark glass `rgba(13, 11, 20, 235)`.
- **Border**: `1px solid rgba(199, 125, 255, 0.4)`.
- **Progress Chunk**: `#C77DFF`.

---

## 4. QPalette Integration

`get_palette("dark")` maps these tokens to Qt's native `QPalette` roles:
- `QPalette.Window` $\rightarrow$ `#0D0B14`
- `QPalette.AlternateBase` $\rightarrow$ `#191624`
- `QPalette.Highlight` $\rightarrow$ `#9D4EDD`
- `QPalette.HighlightedText` $\rightarrow$ `#FFFFFF`

This guarantees that third-party Qt dialogs and custom components inherit the Cyberpunk Purple theme automatically without requiring manual CSS rules.
