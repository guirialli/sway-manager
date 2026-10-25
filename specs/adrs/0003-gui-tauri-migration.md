# ADR 0003: Migração SwayManagerGui PySide6 → Rust + Tauri v2

- **Status**: Proposed
- **Date**: 2026-10-24
- **Authors**: SwayManager Core Team

## Context & Problem Statement

A GUI (`SwayManagerGUI`) é hoje PySide6 compilada via Nuitka; o daemon já foi portado
para Rust (`daemon/`) com paridade byte-a-byte. Manter duas stacks (Qt + Rust) duplica
build, testes e dependências de sistema. O Tauri v2 permite frontend web + backend Rust,
reaproveitando o daemon como fonte de verdade via `invoke`.

## Decision Drivers

1. **Stack única Rust**: lógica de sistema (sysfs, swaymsg, grim/slurp, ddcutil, cliphist)
   já existe testada no daemon; expor como `#[tauri::command]` elimina duplicação Python.
2. **Tema preservado**: tokens Cyberpunk (ver `specs/gui-theme-tokens.md`) viram variáveis
   CSS — replicação visual 1:1 sem QSS gerado em runtime.
3. **Isolamento de processo mantido**: cada janela Tauri = subprocesso efêmero lançado
   pelo daemon (`exit_code: 0` + spawn, como `GuiServerHandler` faz hoje).
4. **Tamanho/distribuição**: `.deb` Tauri 2–10 MB vs Nuitka (dezenas de MB + Qt).

## Decision Outcome

Migrar janela a janela, começando pelas OSDs (menor superfície), depois popups,
depois ConfigCenter/portal/freeze-overlay (maior risco, ver abaixo).
Frontend: HTML/TS mínimo (sem framework pesado) + `@tauri-apps/api/core`.
Comandos `invoke` espelham os repos do daemon (`battery_status`, `theme_toggle`, …);
a GUI nunca toca `/sys` ou `swaymsg` direto.

Riscos assumidos (ver `specs/tauri-v2.md` §6):

- **WebKitGTK DMABUF no Sway**: setar `WEBKIT_DISABLE_DMABUF_RENDERER=1` no `main()`
  antes de iniciar o Tauri; fallback `WEBKIT_DISABLE_COMPOSITING_MODE=1`.
- **Freeze overlay multimonitor**: grim+slurp continuam via comando Rust retornando PNG
  (`tauri::ipc::Response` binário); seleção por drag vira canvas fullscreen por monitor —
  validar transparência/posicionamento no Sway antes de remover o overlay Qt.
- **Portal chooser**: processo modal invocado pelo `xdg-desktop-portal-wlr`; manter
  contrato `output:<n>`/`window:<id>` em stdout e startup < 300 ms — medir contra o Qt.
- **Sem layer-shell**: OSDs viram janelas normais always-on-top; validar no Sway.

## Ordem de porte

1. `VolumeOSD` / `BrightnessOSD` (comando único + timer).
2. `BrightnessPopup`, `MonitorSwapWindow` (estado + confirmação 15 s).
3. `WallpaperPickerWindow` (grid + thumbnails via `Response` binário).
4. `ConfigCenterWindow` (6 páginas) — maior esforço.
5. `PortalDialog` + `FreezeSelectionOverlay` — risco Wayland, por último.

## Consequences

### Positive

- Um build Rust (`cargo`), uma suite (`cargo test`), tipos fim-a-fim via serde.
- CSS estático versionável em vez de QSS interpolado em runtime.

### Negative

- Dependências WebKitGTK no install; frontend TS a manter.
- Paridade visual exige conferência screenshot lado a lado por janela.

### Testes de guarda (regressão)

- `tests/test_gui_*.py` (Python, novos nesta migração) travam comportamento atual e
  servem de oráculo para os testes Rust/Tauri equivalentes.

## Fase 1 executada (2026-10-26)

Crate `gui/src-tauri` (`sway-manager-gui`): `commands::{volume_osd, brightness_osd}`
sobre `sway_manager_daemon::repos::audio`, 6 testes Rust verdes, clippy 0.
Frontend: `gui/src/routes/osd.html` (tokens CSS dark/light, sem framework).
Fase 2 = `tauri = "2"` + `invoke_handler` + `WEBKIT_DISABLE_DMABUF_RENDERER=1`
(requer `libwebkit2gtk-4.1-dev` no host).
