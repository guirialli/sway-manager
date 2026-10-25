# Tauri v2 — Referência para migração do SwayManagerGui

Levantado em 2026-10-24 de `v2.tauri.app` (Tauri Core `2.x`, estável desde 02/10/2024;
bundler `2.10.x`). Verificar versão atual em `https://v2.tauri.app/` antes de codar.

## 1. Arquitetura

- Core Rust + WebView do SO via **WRY** + janelas/event loop via **Tao** (derivado do Winit).
- **Linux = WebKitGTK API 4.1** (GTK 3 + libsoup3). Windows = WebView2 (Chromium),
  macOS/iOS = WKWebView. Sem Chromium empacotado (≠ Electron).
- v2 separa **janela nativa** (`tauri::Window`) de **container web** (`tauri::Webview`);
  para 1 webview por janela usar o unificado **`tauri::WebviewWindow`** /
  **`WebviewWindowBuilder`**.
- Modelo de segurança ACL: **Permissions + Scopes + Capabilities** em JSON/TOML em
  `src-tauri/capabilities/*.json`, vinculando janelas/webviews a comandos core/plugins.
  O `allowlist` do v1 foi removido.
- IPC: `invoke` assíncrono tipado; streaming via `tauri::ipc::Channel`;
  binário bruto via `tauri::ipc::Response`.
  [Guia Rust↔JS](https://v2.tauri.app/develop/calling-rust/)

## 2. Requisitos Linux (Debian/Ubuntu)

Pacotes: `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev
libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
[fonte](https://v2.tauri.app/start/prerequisites/#linux) ·
[Node](https://v2.tauri.app/start/prerequisites/#nodejs) (LTS ≥ 20.10 p/ ferramental)

MSRV: Tauri Core ≥ Rust 1.77.2; plugins recentes (`single-instance`, `notification`
v2.2+) pedem **Rust ≥ 1.90**. Frontend agnóstico (Vite/React/Vue/Svelte/vanilla/WASM);
cliente JS unificado `@tauri-apps/api/core`.

## 3. Comandos IPC (`invoke`)

Backend — `#[tauri::command]`, registro em `.invoke_handler(tauri::generate_handler![...])`,
args via `serde::Deserialize` (frontend `camelCase` → Rust `snake_case`),
`Result<T, E: Serialize>` rejeita a Promise no erro.

```rust
#[tauri::command]
fn battery_status() -> Result<String, String> { /* ... */ }
```

Frontend:

```ts
import { invoke } from '@tauri-apps/api/core';
const status = await invoke<string>('battery_status');
```

[Erros e buffers](https://v2.tauri.app/develop/calling-rust/)

## 4. Multi-window (`WebviewWindow`)

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};
WebviewWindowBuilder::new(app, "config-center", WebviewUrl::App("index.html".into()))
    .title("Config Center").inner_size(920.0, 620.0).build()?;
```

```ts
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
new WebviewWindow('monitor', { url: 'index.html#/monitor', title: 'Monitor' });
```

Recursos Linux: `transient_for(parent)`, `default_vbox()` (acesso ao `gtk::Box` interno).
[changelog v2](https://v2.tauri.app/blog/tauri-20/#changelog)

## 5. Tray / notificações / instância única

- **Tray**: core, feature `tray-icon`; Linux precisa `libayatana-appindicator3-dev`.
  Módulos `tauri::tray` + `tauri::menu` (Click/DoubleClick/Enter/Move/Leave).
  [fonte](https://v2.tauri.app/learn/system-tray/)
- **Notificações**: plugin `tauri-plugin-notification` (`@tauri-apps/plugin-notification`);
  requer capability `"notification:default"`.
  [fonte](https://v2.tauri.app/plugin/notification/)
- **Single-instance**: plugin `tauri-plugin-single-instance` (**registrar primeiro** no
  builder); no Linux via D-Bus `org.{id}.SingleInstance`, 2ª instância encaminha
  `args`+`cwd` e sai.
  [fonte](https://v2.tauri.app/plugin/single-instance/)

## 6. ⚠️ Wayland/Sway — armadilhas conhecidas

- **DMABUF bug** (NVIDIA/wlroots): tela branca/preta no WebKitGTK. Fix oficial no topo
  do `main()`, antes de GTK/Tauri:
  `std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");`
  fallback: `WEBKIT_DISABLE_COMPOSITING_MODE=1`.
  [fonte](https://v2.tauri.app/develop/debug/linux-graphics/)
- **Sem layer-shell**: Tauri = janelas GTK3 normais. Ancorar como dock/panel/overlay
  exclusivo exige integração custom `gtk-layer-shell`.
- OSDs frameless-on-top (`StaysOnTopHint`/`FramelessWindowHint`) mapeiam para janelas
  decoradas-false + always-on-top — validar no Sway.

## 7. Empacotamento Linux

- `.deb`: `cargo tauri build -b deb` (2–10 MB, deps dinâmicas do sistema).
  Compilar na base mais antiga suportada (glibc).
  [fonte](https://v2.tauri.app/distribute/debian/)
- `.AppImage`: `cargo tauri build -b appimage` via linuxdeploy+GTK (~70 MB).
  [fonte](https://v2.tauri.app/distribute/appimage/)
