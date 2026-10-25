//! Binário `SwayManagerGUI` — entrypoint gráfico (Tauri v2 real).
//!
//! Paridade CLI com `src/gui_main.py::run_gui_app`: sem args imprime a ajuda
//! em stdout e sai com 1; comando desconhecido vai para stderr com exit 1.
//! Como o Python, NUNCA encaminha ao daemon (o daemon é quem spawna este
//! binário — encaminhar de volta seria loop). Cada comando abre sua
//! `WebviewWindow` com a rota HTML portada (`gui/src/routes/*.html`).

mod tauri_runtime;
use sway_manager_gui::domains::osd::singleton as osd_singleton;

use tauri::{Manager as _, WebviewUrl, WebviewWindowBuilder};

const HELP: &str = "SwayManagerGUI - Módulo de Interface Gráfica\nUso: SwayManagerGUI <settings|monitor|wallpaper|osd|portal|polkit> [opções]";

/// Comandos aceitos por `run_gui_app` (`src/gui_main.py:13-22`).
fn is_gui_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "settings" | "config" | "config-center" | "monitor" | "wallpaper" | "osd" | "brilho" | "brightness" | "screenshot" | "portal" | "polkit"
    )
}

/// `app_id` Wayland por comando (paridade Qt `setDesktopFileName`).
///
/// O Tauri não expõe o `app_id` GTK por janela (`enableGTKAppId` usa o
/// `identifier` único do config). Como cada rota roda num processo
/// próprio, setamos o `prgname` GLib — o GTK usa-o como fallback do
/// `app_id` Wayland (provado: binário renomeado muda o `app_id`).
/// Rotas sem paridade Python mantêm o basename do executável.
fn prgname_for(cmd: &str) -> Option<&'static str> {
    match cmd {
        // `windows/config_center.py:58`.
        "settings" | "config" | "config-center" => Some("sway-manager"),
        // `windows/wallpaper_picker.py:35`.
        "wallpaper" => Some("sway.apps.wallpaper-picker"),
        // Contrato polkit (§3.4): `for_window [app_id="org.swaymanager.PolkitAuth"]`.
        "polkit" => Some("org.swaymanager.PolkitAuth"),
        _ => None,
    }
}

/// Rota HTML + título + geometria por comando (paridade Qt).
fn window_spec(cmd: &str, sub: Option<&str>) -> (&'static str, &'static str, f64, f64, bool) {
    match cmd {
        "settings" | "config" | "config-center" => ("routes/settings/config-center.html", "SwayManager Control Center", 920.0, 620.0, false),
        "monitor" => ("routes/settings/monitor.html", "Monitores", 520.0, 320.0, false),
        "wallpaper" => ("routes/settings/wallpaper.html", "Seletor de Papel de Parede", 880.0, 560.0, false),
        "portal" => ("routes/system/portal.html", "Compartilhar Tela", 640.0, 480.0, false),
        "polkit" => ("routes/system/polkit.html", "Autenticação necessária", 480.0, 320.0, false),
        "screenshot" if sub == Some("area") => ("routes/system/freeze.html", "Selecionar Área", 1920.0, 1080.0, true),
        "screenshot" => ("routes/system/freeze.html", "Captura de Tela", 1920.0, 1080.0, true),
        "brilho" | "brightness" if sub.is_none() || sub == Some("popup") => {
            ("routes/osd/brightness-popup.html", "Brilho", 320.0, 60.0, true)
        }
        _ => ("routes/osd/osd.html", "OSD", 300.0, 200.0, true),
    }
}

/// Outputs ativos para overlays (um por tela, paridade Qt).
fn overlay_outputs() -> Vec<(String, f64, f64)> {
    let runner = sway_manager_daemon::process::SystemRunner;
    let screens = sway_manager_gui::domains::system::portal_freeze::freeze_screens(
        &runner as &dyn sway_manager_daemon::process::CommandRunner,
    );
    if screens.is_empty() {
        return vec![("".to_string(), 1920.0, 1080.0)];
    }
    screens
        .into_iter()
        .map(|s| (s.name, f64::from(s.geometry.w), f64::from(s.geometry.h)))
        .collect()
}

/// Captura `grim -o` de TODOS os outputs antes de abrir janelas.
/// Paridade `load_frozen_screens`: cada overlay fotografando após abrir
/// captura os outros overlays (transparentes = preto).
fn prefreeze_all_outputs() {
    let runner = sway_manager_daemon::process::SystemRunner;
    prefreeze_with(&runner as &dyn sway_manager_daemon::process::CommandRunner);
}

/// Núcleo testável: `grim -o <nome> /tmp/sway-manager-freeze-<nome>.png`
/// para cada output ativo (paridade `load_frozen_screens`).
fn prefreeze_with(runner: &dyn sway_manager_daemon::process::CommandRunner) {
    for screen in sway_manager_gui::domains::system::portal_freeze::freeze_screens(runner) {
        let dst = format!("/tmp/sway-manager-freeze-{}.png", screen.name);
        let _ = sway_manager_gui::domains::system::portal_freeze::freeze_capture(runner, &screen.name, &dst);
    }
}

/// Poll `swaymsg -t get_tree` até o título existir (até ~2s). Evita a race
/// onde o `move ... to output` roda antes da propagação Wayland do título
/// e falha silencioso (ambas as janelas ficam no output focado).
fn wait_for_title(runner: &dyn sway_manager_daemon::process::CommandRunner, title: &str) {
    for _ in 0..20 {
        let tree = runner.run("swaymsg", &["-t", "get_tree"]);
        if tree.succeeded() && tree.stdout.contains(title) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// `true` se a janela `title` está no output `output` (parse simples da
/// tree: acha o bloco `"name":"<title>"` e volta até o `"type":"output"`
/// mais próximo para ler seu nome).
fn window_on_output(
    runner: &dyn sway_manager_daemon::process::CommandRunner,
    title: &str,
    output: &str,
) -> bool {
    let tree = runner.run("swaymsg", &["-t", "get_tree"]);
    if !tree.succeeded() {
        return false;
    }
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&tree.stdout) else {
        return false;
    };
    fn find(node: &serde_json::Value, title: &str, output: &mut String) -> bool {
        if node.get("type").and_then(|t| t.as_str()) == Some("output")
            && let Some(name) = node.get("name").and_then(|n| n.as_str())
        {
            *output = name.to_string();
        }
        if node.get("name").and_then(|n| n.as_str()) == Some(title) {
            return true;
        }
        for key in ["nodes", "floating_nodes"] {
            if let Some(children) = node.get(key).and_then(|c| c.as_array()) {
                for child in children {
                    let mut out = output.clone();
                    if find(child, title, &mut out) {
                        *output = out;
                        return true;
                    }
                }
            }
        }
        false
    }
    let mut found_output = String::new();
    find(&json, title, &mut found_output) && found_output == output
}

/// Decide se o processo deve encerrar num evento de janela.
///
/// Regressão: sem o `exit(0)` no `Destroyed` da última janela, aplicar no
/// picker (ou fechar no X) deixava o processo órfão com frame fantasma.
fn should_exit_on_window_event(event: &tauri::WindowEvent, no_windows_left: bool) -> bool {
    matches!(event, tauri::WindowEvent::Destroyed) && no_windows_left
}

/// Query do OSD: `osd volume up` → `?kind=volume&action=up` (paridade
/// `VolumeOSD(action)` — sem `action` o OSD só exibe, nunca ajusta).
fn osd_query(sub: Option<&str>, action: Option<&str>) -> String {
    let kind = match sub {
        Some("brilho") | Some("brightness") => "brightness",
        Some("volume") => "volume",
        _ => return String::new(),
    };
    match action {
        Some(a) => format!("?kind={kind}&action={a}"),
        None => format!("?kind={kind}"),
    }
}

/// Lê o stdin pipado (fontes dmenu do xdpw); TTY = `None` (não bloqueia).
fn piped_stdin() -> Option<String> {
    use std::io::{IsTerminal as _, Read as _};
    if std::io::stdin().is_terminal() {
        return None;
    }
    let mut content = String::new();
    std::io::stdin().read_to_string(&mut content).ok()?;
    let trimmed = content.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

/// Percent-encode mínimo p/ query (`decodeURIComponent` no JS).
fn percent_encode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
/// Loop servidor do OSD singleton: cada press executa a ação e emite o
/// estado (`osd-update`) para a janela viva atualizar in-place.
fn osd_singleton_serve(handle: &tauri::AppHandle, kind: &'static str) {
    use tauri::Emitter as _;
    // SAFETY: getuid é async-signal-safe, sem estado.
    let uid = unsafe { libc::getuid() };
    let path = osd_singleton::socket_path(kind, uid, None);
    let Ok(listener) = osd_singleton::bind_server(&path) else {
        return;
    };
    let runner = sway_manager_daemon::process::SystemRunner;
    for stream in listener.incoming().flatten() {
        let Some(action) = osd_singleton::read_action(stream) else {
            continue;
        };
        if kind == "volume" {
            let act = match action.as_str() {
                "up" => sway_manager_gui::VolumeAction::Up,
                "down" => sway_manager_gui::VolumeAction::Down,
                "mute" => sway_manager_gui::VolumeAction::Mute,
                _ => sway_manager_gui::VolumeAction::Unknown,
            };
            if let Ok(state) = sway_manager_gui::volume_osd(&runner, act) {
                // Broadcast (não `emit_to("main")`): o `listen` do frontend
                // registra target `Any` e só recebe broadcast `emit`.
                let _ = handle.emit("osd-update", state);
            }
        } else {
            let act = match action.as_str() {
                "up" => sway_manager_gui::BrightnessAction::Up,
                "down" => sway_manager_gui::BrightnessAction::Down,
                _ => sway_manager_gui::BrightnessAction::Unknown,
            };
            if let Ok(percent) = sway_manager_gui::brightness_osd(&runner, act) {
                let _ = handle.emit("osd-update", percent);
            }
        }
    }
}

/// Janela da rota (`action` = `args[3]`, ação OSD `up`|`down`|`mute`).
fn run_window(cmd: &str, sub: Option<&str>, action: Option<&str>) {
    let (route, title, width, height, frameless) = window_spec(cmd, sub);
    let query: String = match cmd {
        "osd" => osd_query(sub, action),
        "brilho" | "brightness" => match sub {
            Some("up") => "?kind=brightness&action=up".to_string(),
            Some("down") => "?kind=brightness&action=down".to_string(),
            _ => String::new(),
        },
        // Deep-link: `SwayManagerGUI settings lightdm` abre direto na página 5.
        "settings" | "config" | "config-center" => match sub {
            Some("monitores" | "monitors" | "0") => "?page=0".to_string(),
            Some("wallpaper" | "papel" | "1") => "?page=1".to_string(),
            Some("aparencia" | "tema" | "theme" | "2") => "?page=2".to_string(),
            Some("bateria" | "energia" | "power" | "3") => "?page=3".to_string(),
            Some("idle" | "suspensao" | "4") => "?page=4".to_string(),
            Some("lightdm" | "login" | "5") => "?page=5".to_string(),
            _ => String::new(),
        },
        // xdpw (dmenu) passa as fontes no stdin e exige o eco EXATO da
        // linha (`strcmp`). Sem repassar, emitíamos `eDP-1` e o xdpw
        // respondia "selected unknown target".
        "portal" => piped_stdin()
            .map(|s| format!("?stdin={}", percent_encode(&s)))
            .unwrap_or_default(),
        _ => String::new(),
    };
    // `screenshot area` abre UMA janela fullscreen por output, cada uma com
    // `?output=<nome>` (paridade `MultiMonitorSelectionController`).
    let overlays: Option<Vec<(String, f64, f64)>> = (cmd == "screenshot").then(overlay_outputs);
    let url = format!("{route}{query}");
    let osd_kind = osd_singleton::osd_kind(cmd, sub);
    tauri::Builder::default()
        .invoke_handler(tauri_runtime::invoke_handler())
        .on_window_event(|window, event| {
            // Paridade Qt: app single-window morre com a janela. Sem isso,
            // `window.close()` (ou o X) deixa o processo órfão com um frame
            // fantasma no sway — e cada uso vira um "deadprocess".
            if should_exit_on_window_event(event, window.app_handle().webview_windows().is_empty()) {
                window.app_handle().exit(0);
            }
        })
        .setup(move |app| {
            if let Some(outputs) = &overlays {
                for (index, (name, w, h)) in outputs.iter().enumerate() {
                    let label = format!("freeze-{index}");
                    let freeze_title = format!("SwayManager Freeze {name}");
                    let url = WebviewUrl::App(format!("{route}?output={name}").into());
                    WebviewWindowBuilder::new(app, &label, url)
                        .title(&freeze_title)
                        .inner_size(*w, *h)
                        .resizable(false)
                        .decorations(false)
                        // OPACA de propósito: janela transparente fullscreen
                        // no WebKitGTK/wlroots apaga (preto puro) a cada
                        // re-composição (hover/foco — reproduzido). O overlay
                        // cobre 100% dos pixels (freeze + scrim), então a
                        // transparência é desnecessária e só traz o bug.
                        .transparent(false)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .fullscreen(true)
                        .build()?;
                }
                // Posiciona DEPOIS do event loop rodar: dentro do `setup` as
                // janelas existem na tree mas ainda não estão configuradas e
                // o `move ... to output` falha silencioso (evidência: manual
                // após 7s funciona, automático no setup não).
                let pins: Vec<(String, String)> = outputs
                    .iter()
                    .map(|(name, _, _)| (format!("SwayManager Freeze {name}"), name.clone()))
                    .collect();
                std::thread::spawn(move || {
                    use sway_manager_daemon::process::CommandRunner as _;
                    let runner = sway_manager_daemon::process::SystemRunner;
                    std::thread::sleep(std::time::Duration::from_millis(800));
                    for (freeze_title, name) in &pins {
                        let criteria = format!("[title=\"{freeze_title}\"]");
                        wait_for_title(&runner, freeze_title);
                        for _ in 0..10 {
                            runner.run("swaymsg", &[criteria.as_str(), "move", "window", "to", "output", name]);
                            if window_on_output(&runner, freeze_title, name) {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(200));
                        }
                        runner.run("swaymsg", &[criteria.as_str(), "fullscreen", "enable"]);
                    }
                });
                return Ok(());
            }
            // Sem compositing não há transparência real: `transparent(true)`
            // apaga o TEXTO das janelas pequenas (OSD mostrava só a barra).
            // Todas desenham fundo opaco próprio no CSS (freeze já era `false`).
            let is_osd = route == "routes/osd/osd.html";
            let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(url.into()))
                .title(title)
                .inner_size(width, height)
                .resizable(!frameless)
                .decorations(!frameless)
                .transparent(false)
                .always_on_top(frameless)
                // OSD nasce invisível e aparece com conteúdo (mata o flash
                // branco de ~150ms entre a janela e o primeiro paint).
                .visible(!is_osd)
                .center();
            if is_osd {
                builder = builder.on_page_load(|window, _| {
                    let _ = window.show();
                });
            }
            if frameless {
                builder = builder.skip_taskbar(true);
                // Trava o tamanho: sem teto, o sway estica frameless
                // (OSD 300x56 virava 300x200 com área preta).
                builder = builder.max_inner_size(width, height);
            } else {
                // Piso do layout fluido (`body min-width/height` no CSS).
                builder = builder.min_inner_size(640.0, 440.0);
            }
            builder.build()?;
            // Servidor singleton: presses seguintes chegam pelo socket, a
            // ação executa aqui e o estado é emitido (`osd-update`) para
            // o frontend atualizar in-place + resetar o auto-close.
            if let Some(kind) = osd_kind {
                let handle = app.handle().clone();
                std::thread::Builder::new()
                    .name(format!("osd-singleton-{kind}"))
                    .spawn(move || osd_singleton_serve(&handle, kind))
                    .ok();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o runtime Tauri");
}

/// Diálogo polkit (contrato §5): `SwayManagerGUI polkit --action-id …
/// --message … --identities <csv> --pipe <fd> --events <fd>`.
///
/// Abre `routes/system/polkit.html?`action_id=&message=&identities= (via
/// `percent_encode` — NUNCA o cookie, fora do argv por contrato); o pipe
/// (fd 3) vira `State<Mutex<File>>` p/ `polkit_submit/cancel` e o fd de
/// eventos (fd 4) é lido numa task assíncrona que re-emite cada linha BE→GUI
/// como evento `polkit-event` p/ a janela `main` (ignoradas se malformadas).
fn run_polkit() {
    use std::io::BufRead as _;
    use std::os::unix::io::FromRawFd as _;
    use tauri::{Emitter as _, Manager as _};

    let args: Vec<String> = std::env::args().skip(2).collect();
    let launch = match sway_manager_gui::domains::system::polkit_dialog::parse_polkit_args(&args) {
        Ok(launch) => launch,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(1);
        }
    };
    // `File::from_raw_fd` toma posse do fd: fecha ao fim do processo.
    // SAFETY: fds herdados do backend (contrato §5); `parse_polkit_args`
    // garante `>= 0`, e fds inválidos falham abaixo (exit 1, fail-secure).
    let write = unsafe { std::fs::File::from_raw_fd(launch.pipe_fd) };
    let read = unsafe { std::fs::File::from_raw_fd(launch.events_fd) };
    let query = format!(
        "?action_id={}&message={}&identities={}",
        percent_encode(&launch.action_id),
        percent_encode(&launch.message),
        percent_encode(&launch.identities.join(",")),
    );
    let state = sway_manager_gui::domains::system::polkit_dialog::PolkitDialogState::initial(&launch);
    let url = format!("routes/system/polkit.html{query}");
    tauri::Builder::default()
        .manage(state)
        .manage(std::sync::Mutex::new(write))
        .invoke_handler(tauri_runtime::invoke_handler())
        .on_window_event(|window, event| {
            if should_exit_on_window_event(event, window.app_handle().webview_windows().is_empty()) {
                window.app_handle().exit(0);
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            // Leitor do canal BE→GUI: termina em EOF/`completed` (o backend
            // fecha o fd ao concluir). Linha malformada => ignorada.
            tauri::async_runtime::spawn(async move {
                let reader = std::io::BufReader::new(read);
                for line in reader.lines() {
                    let Ok(line) = line else { break };
                    let Some(event) = sway_manager_gui::domains::system::polkit_dialog::gui_event_from_json(&line) else {
                        continue;
                    };
                    let done = matches!(event, sway_manager_gui::domains::system::polkit_dialog::GuiEvent::Completed(_));
                    let payload = event.payload();
                    if handle.emit_to("main", "polkit-event", payload).is_err() {
                        break;
                    }
                    if done {
                        break;
                    }
                }
            });
            WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App(url.into()))
                .title("Autenticação necessária")
                .inner_size(480.0, 320.0)
                .resizable(false)
                .decorations(false)
                .transparent(false)
                .always_on_top(true)
                .center()
                .min_inner_size(480.0, 320.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o runtime Tauri");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("{HELP}");
        std::process::exit(1);
    }
    let cmd = args[1].to_lowercase();
    if !is_gui_command(&cmd) {
        eprintln!("Comando GUI '{cmd}' não reconhecido pelo SwayManagerGUI.");
        std::process::exit(1);
    }
    // Fix Wayland/Sway documentado (specs/tauri-v2.md §6): o renderizador
    // DMABUF do WebKitGTK quebra em compositores wlroots — desliga antes
    // de qualquer init GTK/Tauri. Nível 2 (COMPOSITING_MODE) é obrigatório:
    // sem ele, qualquer re-composição (hover/foco) apaga janelas
    // transparentes fullscreen (overlay vira preto puro — reproduzido).
    // Vale para TODAS as rotas: sem as vars, até o OSD renderiza preto.
    // Efeito colateral: `transparent(true)` vira preto opaco — OSD/popup
    // desenham fundo opaco próprio (`#pill` cobre a janela de 200px).
    // SAFETY: single-thread aqui (início do main, antes de spawnar threads).
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }
    // `app_id` Wayland = `prgname` GLib (fallback GTK sem `enableGTKAppId`).
    // Antes de qualquer init GTK/Tauri — depois o `GtkApplication` já
    // fixou o id e a troca não reflete no compositor.
    if let Some(name) = prgname_for(&cmd) {
        glib::set_prgname(Some(name));
    }
    let sub = args.get(2).map(|s| s.to_lowercase());
    let action = args.get(3).map(|s| s.to_lowercase());
    // Paridade `load_frozen_screens`: captura TODOS os outputs ANTES de
    // abrir qualquer janela — se cada overlay fotografar depois de abrir,
    // um fotografa o outro (transparente = preto).
    if cmd == "screenshot" && matches!(sub.as_deref(), Some("area") | Some("-g")) {
        prefreeze_all_outputs();
    }
    // Paridade `handle_screenshot`: full/window capturam direto (sem janela);
    // area abre o overlay de seleção. `screenshot` bare = full (Qt: default FULL).
    if cmd == "screenshot" && !matches!(sub.as_deref(), Some("area") | Some("-g")) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let runner = sway_manager_daemon::process::SystemRunner;
        let result = match sub.as_deref() {
            Some("window") | Some("-w") => sway_manager_gui::screenshot_window(&runner, &home),
            _ => sway_manager_gui::screenshot_full(&runner, &home),
        };
        match result {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Screenshot falhou: {err}");
                std::process::exit(1);
            }
        }
        return;
    }
    // Desafio polkit: argv próprio (fd herdados), fora do `run_window`.
    if cmd == "polkit" {
        run_polkit();
        return;
    }
    // OSD singleton: se já há janela do kind, notifica (executa + atualiza
    if let Some(kind) = osd_singleton::osd_kind(&cmd, sub.as_deref()) {
        // SAFETY: getuid é async-signal-safe, sem estado.
        let uid = unsafe { libc::getuid() };
        let path = osd_singleton::socket_path(kind, uid, None);
        let act = action.as_deref().unwrap_or("unknown");
        match osd_singleton::try_notify(&path, act) {
            Ok(true) => return,
            Ok(false) => {}
            Err(err) => eprintln!("[osd] aviso: singleton indisponível: {err}"),
        }
    }
    run_window(&cmd, sub.as_deref(), action.as_deref());
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_run_gui_app_commands() {
        for cmd in ["settings", "config", "config-center", "monitor", "wallpaper", "osd", "brilho", "brightness", "screenshot", "portal", "polkit"] {
            assert!(is_gui_command(cmd), "{cmd}");
        }
    }

    #[test]
    fn rejects_unknown_commands() {
        for cmd in ["", "daemon", "battery", "frobnicar"] {
            assert!(!is_gui_command(cmd), "{cmd}");
        }
    }

    #[test]
    fn window_specs_match_qt_geometry() {
        assert_eq!(window_spec("settings", None).0, "routes/settings/config-center.html");
        assert_eq!(window_spec("monitor", None).0, "routes/settings/monitor.html");
        assert_eq!(window_spec("wallpaper", None).0, "routes/settings/wallpaper.html");
        assert_eq!(window_spec("brilho", None).0, "routes/osd/brightness-popup.html");
        assert_eq!(window_spec("brilho", Some("popup")).0, "routes/osd/brightness-popup.html");
        assert_eq!(window_spec("brilho", Some("up")).0, "routes/osd/osd.html");
        assert_eq!(window_spec("screenshot", Some("area")).0, "routes/system/freeze.html");
        assert_eq!(window_spec("portal", None).0, "routes/system/portal.html");
        assert_eq!(window_spec("polkit", None), ("routes/system/polkit.html", "Autenticação necessária", 480.0, 320.0, false));
        assert!(!window_spec("settings", None).4);
        assert!(window_spec("screenshot", Some("area")).4);
    }

    #[test]
    fn window_titles_match_python_windows() {
        // Paridade `config_center.py:59` e `wallpaper_picker.py:29` —
        // regras sway (`for_window [title=...]`) dependem deles.
        assert_eq!(window_spec("settings", None).1, "SwayManager Control Center");
        assert_eq!(window_spec("config-center", None).1, "SwayManager Control Center");
        assert_eq!(window_spec("wallpaper", None).1, "Seletor de Papel de Parede");
    }

    #[test]
    fn prgname_matches_python_desktop_file_names() {
        // Paridade `setDesktopFileName`: o `app_id` Wayland é o `prgname`
        // GLib (fallback GTK). Rotas sem paridade → basename do binário.
        assert_eq!(prgname_for("settings"), Some("sway-manager"));
        assert_eq!(prgname_for("config"), Some("sway-manager"));
        assert_eq!(prgname_for("config-center"), Some("sway-manager"));
        assert_eq!(prgname_for("wallpaper"), Some("sway.apps.wallpaper-picker"));
        assert_eq!(prgname_for("monitor"), None);
        assert_eq!(prgname_for("osd"), None);
        assert_eq!(prgname_for("portal"), None);
        assert_eq!(prgname_for("polkit"), Some("org.swaymanager.PolkitAuth"));
        assert_eq!(prgname_for("screenshot"), None);
    }

    #[test]
    fn prefreeze_captures_each_active_output() {

        use sway_manager_daemon::process::{CmdOutput, CommandRunner};
        use std::sync::{Arc, Mutex};
        #[derive(Default)]
        struct Rec {
            calls: Arc<Mutex<Vec<String>>>,
        }
        impl CommandRunner for Rec {
            fn run(&self, program: &str, args: &[&str]) -> CmdOutput {
                self.calls.lock().unwrap().push(format!("{program} {}", args.join(" ")));
                if program == "swaymsg" {
                    return CmdOutput::success(r#"[{"name":"eDP-1","active":true,"scale":1.0,"rect":{"x":0,"y":0,"width":1920,"height":1080}},{"name":"HDMI-A-1","active":true,"scale":1.0,"rect":{"x":1920,"y":0,"width":1920,"height":1080}}]"#);
                }
                CmdOutput::failure(1, "", "no file")
            }
            fn spawn_detached(&self, _p: &str, _a: &[&str]) {}
            fn which(&self, _b: &str) -> Option<String> {
                None
            }
        }
        let runner = Rec::default();
        prefreeze_with(&runner);
        let calls = runner.calls.lock().unwrap();
        assert!(calls.iter().any(|c| c.contains("grim -o eDP-1 /tmp/sway-manager-freeze-eDP-1.png")), "{calls:?}");
        assert!(calls.iter().any(|c| c.contains("grim -o HDMI-A-1 /tmp/sway-manager-freeze-HDMI-A-1.png")), "{calls:?}");
    }

    #[test]
    fn exit_when_last_window_destroyed() {
        // Regressão: picker não morria ao aplicar (frame fantasma + deadprocess).
        assert!(should_exit_on_window_event(&tauri::WindowEvent::Destroyed, true));
    }

    #[test]
    fn no_exit_when_windows_remain() {
        // `screenshot area` tem N overlays: destruir um não pode matar o app.
        assert!(!should_exit_on_window_event(&tauri::WindowEvent::Destroyed, false));
    }

    #[test]
    fn no_exit_on_non_destroy_events() {
        // Foco/resize/mover nunca encerram, mesmo sem janelas (estado transitório).
        assert!(!should_exit_on_window_event(&tauri::WindowEvent::Focused(true), true));
        assert!(!should_exit_on_window_event(&tauri::WindowEvent::Focused(false), true));
    }

    #[test]
    fn osd_query_carries_action() {
        // Regressão: `osd volume up` sem `action=` só exibia, nunca ajustava.
        assert_eq!(osd_query(Some("volume"), Some("up")), "?kind=volume&action=up");
        assert_eq!(osd_query(Some("volume"), Some("down")), "?kind=volume&action=down");
        assert_eq!(osd_query(Some("volume"), Some("mute")), "?kind=volume&action=mute");
        assert_eq!(osd_query(Some("brilho"), Some("up")), "?kind=brightness&action=up");
    }

    #[test]
    fn osd_query_without_action_shows_state_only() {
        assert_eq!(osd_query(Some("volume"), None), "?kind=volume");
        assert_eq!(osd_query(Some("brilho"), None), "?kind=brightness");
        assert!(osd_query(None, Some("up")).is_empty());
    }

    #[test]
    fn percent_encode_roundtrips_xdpw_line() {
        // Regressão: o xdpw exige o eco EXATO (`strcmp`); o encode não
        // pode corromper a linha `Monitor: ... (...)` no caminho.
        let line = "Monitor: eDP-1 AU Optronics B153UAN03.0  (eDP-1)";
        let enc = percent_encode(line);
        assert!(!enc.contains([' ', ':', '(', ')']), "{enc}");
        // decode manual espelhando `decodeURIComponent`:
        let mut dec = String::new();
        let bytes = enc.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
                dec.push(u8::from_str_radix(hex, 16).unwrap() as char);
                i += 3;
            } else {
                dec.push(bytes[i] as char);
                i += 1;
            }
        }
        assert_eq!(dec, line);
    }
}
