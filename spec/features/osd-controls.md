# Feature Spec: On-Screen Display (OSD) & Popups (`osd`, `brightness`)

## 1. Overview & Objective
Provides visual feedback overlays for brightness and volume adjustments. Renders translucent, auto-hiding OSD windows when hardware keys or CLI commands trigger volume or screen brightness updates.

---

## 2. Scope & Responsibilities

### In Scope
- Adjusting screen brightness via `brightnessctl` repository.
- Adjusting audio output volume and mute state via Mixer audio repository (`amixer` / `wpctl`).
- Displaying auto-hiding PySide6 translucent overlay widgets (`BrightnessOSD`, `VolumeOSD`).
- Displaying interactive slider popups (`BrightnessPopup`).

---

## 3. Contracts & Interfaces

### CLI Commands
```bash
SwayManager osd brilho [up|down|popup]
SwayManager osd volume [up|down|mute]
SwayManager brightness [up|down|popup]
```

### OSD Actions & Steps
| Command | Action Argument | Default Adjustment Step |
|---|---|---|
| `osd brilho` | `up` | +5% brightness |
| `osd brilho` | `down` | -5% brightness |
| `osd volume` | `up` | +5% volume |
| `osd volume` | `down` | -5% volume |
| `osd volume` | `mute` | Toggle audio mute |

---

## 4. Architecture & Component Flow

```mermaid
graph TD
    Key[Hardware Key / Sway Config] --> CLI[SwayManager osd brilho up]
    CLI --> UC[SetBrightnessUseCase / AdjustVolumeUseCase]
    UC --> Repo[BrightnessctlRepository / MixerAudioRepository]
    CLI --> OSD[BrightnessOSD / VolumeOSD Widget]
    OSD --> Timer[Auto-Hide QTimer (2s)]
    Timer --> Close[Close & Memory Trim]
```

---

## 5. UI / UX Specifications
- **Window Hints**: `FramelessWindowHint`, `WindowStaysOnTopHint`, `WA_TranslucentBackground`.
- **Auto-Dismiss**: OSD overlays auto-close after 2000 ms of inactivity.
- **Styling**: Apple HIG translucent background (`rgba(30, 30, 32, 220)`), progress bar indicator.

---

## 6. Acceptance Criteria & Verification
- [ ] `SwayManager osd brilho up` increases brightness and renders the OSD overlay.
- [ ] `SwayManager osd volume mute` toggles audio mute and updates OSD icon.
- [ ] OSD overlay automatically closes after 2 seconds without stealing window focus.
