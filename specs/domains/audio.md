# Domain Spec: Audio Subsystem

## 1. Overview & Responsibilities

The **Audio Domain** manages sound volume level adjustments, output sink muting, and visual feedback via transient On-Screen Display (OSD) popups.

---

## 2. Component Mapping

| Layer | Component | File Path | Responsibility |
|---|---|---|---|
| Domain | `IAudioRepository` | `src/domain/audio/repositories.py` | Abstract interface for volume queries and adjustments |
| Domain | `VolumeLevel` | `src/domain/audio/value_objects.py` | Value object encapsulating integer volume percentage and mute flag |
| Application | `AdjustVolumeUseCase` | `src/application/audio/adjust_volume_use_case.py` | Use case orchestrating volume changes and notifications |
| Infrastructure | `MixerAudioRepository` | `src/infrastructure/audio/mixer_audio_repository.py` | Concrete adapter wrapping `pactl`, `wpctl`, or `pamixer` |
| Presentation | `VolumeOSD` | `src/presentation/gui/osd/volume_osd.py` | PySide6 transient OSD popup window |

---

## 3. Interfaces & Contracts

### CLI Triggers
```bash
SwayManager osd volume [action]
```

### Supported Actions:
- `up` / `increase`: Increases master volume by 5%.
- `down` / `decrease`: Decreases master volume by 5%.
- `mute` / `toggle-mute`: Toggles sink mute state.

---

## 4. Execution & Flow

```mermaid
sequenceDiagram
    participant User as Sway Keybinding
    participant CLI as CLI Router / Handler
    participant UC as AdjustVolumeUseCase
    participant Repo as MixerAudioRepository
    participant Binary as pactl / wpctl / pamixer
    participant OSD as VolumeOSD (PySide6)

    User->>CLI: SwayManager osd volume up
    CLI->>UC: execute(action="up")
    UC->>Repo: adjust_volume(+5)
    Repo->>Binary: pactl set-sink-volume @DEFAULT_SINK@ +5%
    UC->>OSD: ApplicationFactory.buildWidget(VolumeOSD)
    OSD->>OSD: Render volume bar & auto-close after 1.5s
```

---

## 5. Failure Modes & Edge Cases

| Condition | System Behavior | Mitigation |
|---|---|---|
| `pactl`/`wpctl` missing | Command catches `FileNotFoundError` | Falls back gracefully, prints warning to stderr |
| Volume exceeds 100% | Clamped by repository / mixer | Safe ceiling limit enforced |
| Headless execution | OSD creation skipped if Qt display unavailable | Safe fallback to CLI text output |
