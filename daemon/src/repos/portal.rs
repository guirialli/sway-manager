//! Screen-share portal contracts — result lines, providers.
//!
//! Mirrors `src/portal/` from the Python implementation 1:1
//! (sem `selection_cache.py`: o replay silencioso de 30 s foi removido —
//! o diálogo sempre abre):
//! `result_writer.py` (identifier guard + `monitor`/`window` lines),
//! `outputs_provider.py` (`swaymsg -t get_outputs -r` parsing) and
//! `windows_provider.py` (sway-version gate + `lswt -j` parsing).
//! The Qt chooser UI stays in Python; this module owns the headless
//! contracts the daemon path can execute. `diagnostics.py` lives here
//! too (`sway --version`, `which pipewire/pw-cli`, `systemctl is-active`).

use crate::process::CommandRunner;

pub const MIN_SWAY_MAJOR: u32 = 1;
pub const MIN_SWAY_MINOR: u32 = 12;
// --- models (mirrors `portal/models.py`) -----------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Monitor,
    Window,
}

impl SourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceType::Monitor => "monitor",
            SourceType::Window => "window",
        }
    }

    pub fn parse(raw: &str) -> Self {
        if raw == "monitor" {
            SourceType::Monitor
        } else {
            SourceType::Window
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalSource {
    pub id: String,
    pub source_type: SourceType,
    pub label: String,
    pub details: String,
    pub is_primary: bool,
    pub is_focused: bool,
    pub raw_label: Option<String>,
}

impl PortalSource {
    pub fn new(
        id: &str,
        source_type: SourceType,
        label: &str,
        details: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            source_type,
            label: label.to_string(),
            details: details.to_string(),
            is_primary: false,
            is_focused: false,
            raw_label: None,
        }
    }

    /// Identity: `raw_label or id`.
    pub fn identity(&self) -> &str {
        self.raw_label.as_deref().unwrap_or(&self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalResult {
    pub source_type: SourceType,
    pub id: String,
    pub raw_label: Option<String>,
}

impl PortalResult {
    pub fn new(source_type: SourceType, id: &str) -> Self {
        Self { source_type, id: id.to_string(), raw_label: None }
    }

    /// Mirrors `PortalResult.__str__`: `raw_label` when set, else `id`.
    pub fn line(&self) -> &str {
        self.raw_label.as_deref().unwrap_or(&self.id)
    }
}

impl std::fmt::Display for PortalResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.line())
    }
}

// --- result_writer (mirrors `portal/result_writer.py`) ----------------------

/// Mirrors `PortalResultWriter.validate_identifier`.
pub fn validate_identifier(identifier: &str) -> bool {
    !identifier.is_empty()
        && !identifier.contains('\n')
        && !identifier.contains('\r')
}

/// Mirrors `write_result`: the single line emitted on stdout, or `None`
/// when there is nothing to emit (cancelled / empty).
pub fn format_result(result: &PortalResult) -> Option<String> {
    let line = result.line();
    if line.is_empty() { None } else { Some(format!("{line}\n")) }
}

/// Mirrors `write_monitor`.
pub fn format_monitor(output_name: &str) -> Option<String> {
    format_result(&PortalResult::new(SourceType::Monitor, output_name))
}

/// Mirrors `write_window`.
pub fn format_window(window_id: &str) -> Option<String> {
    format_result(&PortalResult::new(SourceType::Window, window_id))
}

// --- outputs provider (mirrors `portal/outputs_provider.py`) ----------------

/// Parse `swaymsg -t get_outputs -r` JSON into active sources.
///
/// Error literals mirror `SwayNotAvailableError` messages 1:1.
pub fn parse_outputs(json_text: &str) -> Result<Vec<PortalSource>, String> {
    if json_text.trim().is_empty() {
        return Err("swaymsg returned empty output".to_string());
    }
    let outputs: Vec<serde_json::Value> =
        serde_json::from_str(json_text).map_err(|err| format!("invalid JSON from swaymsg: {err}"))?;
    let mut sources = Vec::new();
    for output in &outputs {
        if !output.get("active").and_then(serde_json::Value::as_bool).unwrap_or(false) {
            continue;
        }
        let Some(name) = output.get("name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let mode = output.get("current_mode");
        let width = mode.and_then(|m| m.get("width")).and_then(serde_json::Value::as_i64).unwrap_or(0);
        let height = mode.and_then(|m| m.get("height")).and_then(serde_json::Value::as_i64).unwrap_or(0);
        let refresh = mode.and_then(|m| m.get("refresh")).and_then(serde_json::Value::as_i64).unwrap_or(0);
        let scale = output.get("scale").and_then(serde_json::Value::as_f64).unwrap_or(1.0);
        let transform = output.get("transform").and_then(serde_json::Value::as_str).unwrap_or("normal");
        let rect = output.get("rect");
        let pos_x = rect.and_then(|r| r.get("x")).and_then(serde_json::Value::as_i64).unwrap_or(0);
        let pos_y = rect.and_then(|r| r.get("y")).and_then(serde_json::Value::as_i64).unwrap_or(0);
        let make = output.get("make").and_then(serde_json::Value::as_str).unwrap_or("").trim();
        let model = output.get("model").and_then(serde_json::Value::as_str).unwrap_or("").trim();
        let serial = output.get("serial").and_then(serde_json::Value::as_str).unwrap_or("").trim();

        let mut label_parts: Vec<&str> = Vec::new();
        if !make.is_empty() {
            label_parts.push(make);
        }
        if !model.is_empty() {
            label_parts.push(model);
        }
        let mut label = if label_parts.is_empty() { name.to_string() } else { label_parts.join(" ") };
        if !serial.is_empty() {
            label = format!("{label} ({serial})");
        }

        let mut details = vec![name.to_string()];
        if width != 0 && height != 0 {
            details.push(format!("{width} \u{d7} {height}"));
        }
        if refresh != 0 {
            details.push(format!("{:.1} Hz", f64::from(refresh as f32) / 1000.0));
        }
        let scale_text = if scale.fract() == 0.0 { format!("{scale:.1}") } else { format!("{scale}") };
        details.push(format!("Escala: {scale_text}"));
        if transform != "normal" {
            details.push(format!("Rota\u{e7}\u{e3}o: {transform}"));
        }
        details.push(format!("Posi\u{e7}\u{e3}o: {pos_x},{pos_y}"));
        let is_primary = output.get("primary").and_then(serde_json::Value::as_bool).unwrap_or(false);
        if is_primary {
            details.push("Principal".to_string());
        }
        sources.push(PortalSource {
            id: name.to_string(),
            source_type: SourceType::Monitor,
            label,
            details: details.join("\n"),
            is_primary,
            is_focused: output.get("focused").and_then(serde_json::Value::as_bool).unwrap_or(false),
            raw_label: None,
        });
    }
    if sources.is_empty() {
        return Err("nenhuma tela ativa encontrada".to_string());
    }
    Ok(sources)
}

/// Run `swaymsg -t get_outputs -r` and parse the active outputs.
pub fn get_outputs(runner: &dyn CommandRunner) -> Result<Vec<PortalSource>, String> {
    let result = runner.run("swaymsg", &["-t", "get_outputs", "-r"]);
    if !result.succeeded() {
        return Err(format!("swaymsg failed (code {}): {}", result.status, result.stderr.trim()));
    }
    parse_outputs(&result.stdout)
}

// --- windows provider (mirrors `portal/windows_provider.py`) ----------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowAvailability {
    pub supported: bool,
    pub reason: Option<String>,
}

/// Mirrors `_parse_sway_version`: `sway version 1.12`, `sway 1.11`,
/// `swayfx ... (based on sway 1.11.0)`.
pub fn parse_sway_version(text: &str) -> Option<(u32, u32)> {
    let lower = text.to_lowercase();
    let bytes = lower.as_bytes();
    // Índices sempre em char boundary (`i += 1` atômico panica em UTF-8).
    let step = |s: &str, i: usize| s[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    let mut i = 0;
    while i < bytes.len() {
        // `starts_with` nunca panica; o slice `[i..i+4]` sim (i+4 pode
        // cortar um char multi-byte).
        if lower[i..].starts_with("sway")
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
            && (i + 4 >= bytes.len() || !bytes[i + 4].is_ascii_alphanumeric())
        {
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if lower[j..].starts_with("version")
                && (j == 0 || !bytes[j - 1].is_ascii_alphanumeric())
            {
                j += "version".len();
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
            }
            let mut k = j;
            while k < bytes.len() && bytes[k].is_ascii_digit() {
                k += 1;
            }
            if k > j && k < bytes.len() && bytes[k] == b'.' {
                let major: u32 = lower[j..k].parse().ok()?;
                let mut m = k + 1;
                while m < bytes.len() && bytes[m].is_ascii_digit() {
                    m += 1;
                }
                if m > k + 1 {
                    let minor: u32 = lower[k + 1..m].parse().ok()?;
                    return Some((major, minor));
                }
            }
            i = j.max(i + 1);
            // `j` pode não ser boundary após saltos ASCII — realinha.
            while i < bytes.len() && !lower.is_char_boundary(i) {
                i += 1;
            }
        } else {
            i += step(&lower, i);
        }
    }
    None
}

pub fn supports_version(version: (u32, u32)) -> bool {
    let (major, minor) = version;
    major > MIN_SWAY_MAJOR || (major == MIN_SWAY_MAJOR && minor >= MIN_SWAY_MINOR)
}

fn sway_version(runner: &dyn CommandRunner) -> Option<(u32, u32)> {
    let result = runner.run("sway", &["--version"]);
    if !result.succeeded() {
        return None;
    }
    parse_sway_version(&result.stdout)
}

fn has_lswt(runner: &dyn CommandRunner) -> bool {
    let result = runner.run("which", &["lswt"]);
    result.succeeded() && !result.stdout.trim().is_empty()
}

/// Mirrors `get_availability` with the exact Python reason literals.
pub fn window_availability(runner: &dyn CommandRunner) -> WindowAvailability {
    let Some(version) = sway_version(runner) else {
        return WindowAvailability {
            supported: false,
            reason: Some(
                "N\u{e3}o foi poss\u{ed}vel detectar a vers\u{e3}o do Sway. A captura de janelas requer Sway 1.12 ou superior.".to_string(),
            ),
        };
    };
    if !supports_version(version) {
        return WindowAvailability {
            supported: false,
            reason: Some(format!(
                "Este compositor \u{e9} baseado no Sway {}.{}. A captura individual de janelas requer Sway 1.12 ou superior.",
                version.0, version.1
            )),
        };
    }
    if !has_lswt(runner) {
        return WindowAvailability {
            supported: false,
            reason: Some(
                "O utilit\u{e1}rio lswt n\u{e3}o est\u{e1} instalado. Instale-o para listar janelas compartilh\u{e1}veis.".to_string(),
            ),
        };
    }
    WindowAvailability { supported: true, reason: None }
}

/// Mirrors `is_supported`.
pub fn windows_supported(runner: &dyn CommandRunner) -> bool {
    sway_version(runner).is_some_and(supports_version)
}

/// Quote bare hex identifiers (`lswt` v2.0.0 emits `"id": abc123`).
fn quote_bare_ids(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let mut matched = false;
        for key in [r#""identifier""#, r#""id""#] {
            if raw[i..].starts_with(key) {
                let mut j = i + key.len();
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b':' {
                    j += 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j < bytes.len() && bytes[j] != b'"' {
                        let mut k = j;
                        while k < bytes.len()
                            && !bytes[k].is_ascii_whitespace()
                            && !matches!(bytes[k], b',' | b'}' | b']')
                        {
                            k += 1;
                        }
                        out.push_str(&raw[i..j]);
                        out.push('"');
                        out.push_str(&raw[j..k]);
                        out.push('"');
                        i = k;
                        matched = true;
                        break;
                    }
                }
            }
        }
        if !matched {
            // Avanço por char: títulos de janela têm UTF-8 arbitrário e
            // `i += 1` atômico panica no meio de chars multi-byte (`π`).
            let ch = raw[i..].chars().next().unwrap_or('\u{FFFD}');
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn window_from_toplevel(toplevel: &serde_json::Value) -> Option<PortalSource> {
    let map = toplevel.as_object()?;
    let id = map
        .get("identifier")
        .or_else(|| map.get("id"))
        .and_then(|v| {
            if let Some(n) = v.as_str() {
                Some(n.to_string())
            } else if v.is_number() {
                Some(v.to_string())
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;
    let app_id = map
        .get("app-id")
        .or_else(|| map.get("app_id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let title = map
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let label = if !title.is_empty() {
        title.clone()
    } else if !app_id.is_empty() {
        app_id.clone()
    } else {
        id.clone()
    };
    let mut details: Vec<String> = Vec::new();
    if !app_id.is_empty() && app_id != title {
        details.push(app_id);
    }
    if let Some(workspace) = map.get("workspace").and_then(|v| v.as_str()) {
        details.push(format!("Workspace {workspace}"));
    }
    Some(PortalSource {
        id,
        source_type: SourceType::Window,
        label,
        details: details.join("\n"),
        is_primary: false,
        is_focused: map
            .get("activated")
            .or_else(|| map.get("focused"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        raw_label: None,
    })
}

fn parse_lswt_json(stdout: &str) -> Option<Vec<PortalSource>> {
    let cleaned = quote_bare_ids(stdout);
    let data: serde_json::Value = serde_json::from_str(&cleaned).ok()?;
    let toplevels: Vec<&serde_json::Value> = match &data {
        serde_json::Value::Array(items) => items.iter().collect(),
        serde_json::Value::Object(map) => map
            .get("toplevels")
            .and_then(|v| v.as_array())
            .map(|items| items.iter().collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let mut sources = Vec::new();
    for toplevel in toplevels {
        if let Some(source) = window_from_toplevel(toplevel) {
            sources.push(source);
        }
    }
    Some(sources)
}

/// Mirrors `_list_windows_with_lswt` (JSON first, `lswt -c iat` fallback).
pub fn list_windows(runner: &dyn CommandRunner) -> Result<Vec<PortalSource>, String> {
    let result = runner.run("lswt", &["-j"]);
    if result.status == 0 && !result.stdout.trim().is_empty() && let Some(sources) = parse_lswt_json(&result.stdout) {
        return Ok(sources);
    }
    let alt = runner.run("lswt", &["-c", "iat"]);
    if alt.status != 0 {
        let stderr = if result.stderr.trim().is_empty() {
            alt.stderr.trim().to_string()
        } else {
            result.stderr.trim().to_string()
        };
        return Err(format!("lswt falhou: {stderr}"));
    }
    let mut sources = Vec::new();
    for line in alt.stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, ',');
        let id = parts.next().map(str::trim).unwrap_or("");
        if id.is_empty() {
            continue;
        }
        let app_id = parts.next().map(str::trim).unwrap_or("");
        let title = parts.next().map(str::trim).unwrap_or("");
        let label = if !title.is_empty() {
            title
        } else if !app_id.is_empty() {
            app_id
        } else {
            id
        };
        let mut details = Vec::new();
        if !app_id.is_empty() && app_id != title {
            details.push(app_id.to_string());
        }
        sources.push(PortalSource {
            id: id.to_string(),
            source_type: SourceType::Window,
            label: label.to_string(),
            details: details.join("\n"),
            is_primary: false,
            is_focused: false,
            raw_label: None,
        });
    }
    Ok(sources)
}

/// Mirrors `get_windows`: `[]` when unsupported, else the `lswt` list.
pub fn get_windows(runner: &dyn CommandRunner) -> Result<Vec<PortalSource>, String> {
    if !window_availability(runner).supported {
        return Ok(Vec::new());
    }
    list_windows(runner).map_err(|err| format!("falha ao consultar janelas: {err}"))
}

// --- diagnostics (mirrors `portal/diagnostics.py`) --------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticsEnv {
    pub session_type: String,
    pub wayland_display: String,
    pub current_desktop: String,
    pub sway_sock: String,
    pub session_desktop: String,
}

impl DiagnosticsEnv {
    pub fn live() -> Self {
        Self {
            session_type: std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
            wayland_display: std::env::var("WAYLAND_DISPLAY").unwrap_or_default(),
            current_desktop: std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
            sway_sock: std::env::var("SWAYSOCK").unwrap_or_default(),
            session_desktop: std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticsReport {
    pub is_wayland: bool,
    pub compositor: String,
    pub supports_window_sharing: bool,
    pub window_sharing_reason: Option<String>,
    pub pipewire_available: bool,
    pub xdg_desktop_portal_active: bool,
    pub xdg_desktop_portal_wlr_active: bool,
    pub session_vars_exported: bool,
    pub errors: Vec<String>,
}

impl DiagnosticsReport {
    pub fn is_ready(&self) -> bool {
        self.is_wayland
            && (self.compositor == "sway" || self.compositor == "swayfx")
            && self.pipewire_available
            && self.xdg_desktop_portal_active
            && self.xdg_desktop_portal_wlr_active
            && self.session_vars_exported
    }
}

fn detect_compositor(runner: &dyn CommandRunner, env: &DiagnosticsEnv) -> String {
    let result = runner.run("sway", &["--version"]);
    if result.succeeded() {
        let text = result.stdout.to_lowercase();
        if text.contains("swayfx") {
            return "swayfx".to_string();
        }
        if text.contains("sway") {
            return "sway".to_string();
        }
    }
    if env.current_desktop.to_lowercase().contains("sway") {
        return "sway".to_string();
    }
    String::new()
}

fn service_active(runner: &dyn CommandRunner, name: &str) -> bool {
    runner.run("systemctl", &["--user", "is-active", name]).succeeded()
}

/// Full diagnostics run with an explicit environment (tests inject it).
pub fn diagnose_with(runner: &dyn CommandRunner, env: &DiagnosticsEnv) -> DiagnosticsReport {
    let is_wayland = env.session_type.to_lowercase() == "wayland" || !env.wayland_display.is_empty();
    let compositor = detect_compositor(runner, env);
    let availability = window_availability(runner);
    let pipewire_available =
        runner.which("pipewire").is_some() || runner.which("pw-cli").is_some();
    let xdg_desktop_portal_active = service_active(runner, "xdg-desktop-portal");
    let xdg_desktop_portal_wlr_active = service_active(runner, "xdg-desktop-portal-wlr");
    let session_vars_exported = !env.wayland_display.is_empty()
        && !env.current_desktop.is_empty()
        && (!env.sway_sock.is_empty()
            || !env.session_type.is_empty()
            || !env.session_desktop.is_empty());

    let mut errors = Vec::new();
    if !is_wayland {
        errors.push("A sess\u{e3}o atual n\u{e3}o utiliza Wayland.".to_string());
    }
    if compositor != "sway" && compositor != "swayfx" {
        errors.push("O compositor atual n\u{e3}o \u{e9} Sway ou SwayFX.".to_string());
    }
    if !pipewire_available {
        errors.push("PipeWire n\u{e3}o est\u{e1} dispon\u{ed}vel.".to_string());
    }
    if !xdg_desktop_portal_active {
        errors.push("O servi\u{e7}o xdg-desktop-portal n\u{e3}o est\u{e1} ativo.".to_string());
    }
    if !xdg_desktop_portal_wlr_active {
        errors.push("O servi\u{e7}o xdg-desktop-portal-wlr n\u{e3}o est\u{e1} ativo.".to_string());
    }
    if !session_vars_exported {
        errors.push(
            "Vari\u{e1}veis de sess\u{e3}o n\u{e3}o foram exportadas para o D-Bus (WAYLAND_DISPLAY ou XDG_CURRENT_DESKTOP podem estar faltando).".to_string(),
        );
    }
    let window_sharing_reason = availability.reason.clone();
    if let Some(reason) = &window_sharing_reason {
        errors.push(reason.clone());
    }

    DiagnosticsReport {
        is_wayland,
        compositor,
        supports_window_sharing: availability.supported,
        window_sharing_reason,
        pipewire_available,
        xdg_desktop_portal_active,
        xdg_desktop_portal_wlr_active,
        session_vars_exported,
        errors,
    }
}

/// Diagnostics against the live process environment.
pub fn diagnose(runner: &dyn CommandRunner) -> DiagnosticsReport {
    diagnose_with(runner, &DiagnosticsEnv::live())
}

/// Mirrors `handle_portal -- status` stdout, byte-for-byte.
pub fn status_text(report: &DiagnosticsReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Wayland: {}\n", if report.is_wayland { "sim" } else { "n\u{e3}o" }));
    out.push_str(&format!(
        "Compositor: {}\n",
        if report.compositor.is_empty() { "n\u{e3}o detectado".to_string() } else { report.compositor.clone() }
    ));
    out.push_str(&format!(
        "Captura de janelas: {}\n",
        if report.supports_window_sharing { "sim" } else { "n\u{e3}o" }
    ));
    out.push_str(&format!("PipeWire: {}\n", if report.pipewire_available { "sim" } else { "n\u{e3}o" }));
    out.push_str(&format!(
        "xdg-desktop-portal: {}\n",
        if report.xdg_desktop_portal_active { "ativo" } else { "inativo" }
    ));
    out.push_str(&format!(
        "xdg-desktop-portal-wlr: {}\n",
        if report.xdg_desktop_portal_wlr_active { "ativo" } else { "inativo" }
    ));
    out.push_str(&format!(
        "Vari\u{e1}veis de sess\u{e3}o: {}\n",
        if report.session_vars_exported { "exportadas" } else { "pendentes" }
    ));
    if !report.errors.is_empty() {
        out.push_str("\nProblemas encontrados:\n");
        for error in &report.errors {
            out.push_str(&format!("  - {error}\n"));
        }
    }
    out.push_str(&format!(
        "\nPronto para compartilhar: {}\n",
        if report.is_ready() { "sim" } else { "n\u{e3}o" }
    ));
    out
}

// --- config installer (mirrors `portal/config_installer.py`) -----------------

/// Merge `entries` into the `[section]` of an INI text, preserving every
/// other section/key (configparser-compatible output for our files).
fn merge_ini_section(existing: &str, section: &str, entries: &[(&str, String)]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut section_seen = false;
    let mut written = vec![false; entries.len()];
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_section {
                for (i, (key, value)) in entries.iter().enumerate() {
                    if !written[i] {
                        out.push(format!("{key} = {value}"));
                        written[i] = true;
                    }
                }
            }
            in_section = trimmed == format!("[{section}]");
            section_seen = section_seen || in_section;
            out.push(line.to_string());
            continue;
        }
        if in_section {
            if let Some((key, _)) = trimmed.split_once('=') {
                let key = key.trim();
                if let Some(i) = entries.iter().position(|(k, _)| *k == key) {
                    out.push(format!("{key} = {}", entries[i].1));
                    written[i] = true;
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    if !section_seen {
        if !out.is_empty() && !out.last().is_some_and(|l| l.trim().is_empty()) {
            out.push(String::new());
        }
        out.push(format!("[{section}]"));
        for (key, value) in entries {
            out.push(format!("{key} = {value}"));
        }
    } else if in_section {
        for (i, (key, value)) in entries.iter().enumerate() {
            if !written[i] {
                out.push(format!("{key} = {value}"));
            }
        }
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

fn backup_path(path: &std::path::Path) -> std::path::PathBuf {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    path.with_extension(format!(
        "{ext}.{secs}.bak",
        ext = path.extension().and_then(|e| e.to_str()).unwrap_or("conf")
    ))
}

/// Install/update the portal configuration files (`portals.conf`,
/// `sway-portals.conf`, xdpw `config`, sway session env) and restart the
/// portal services. Returns the human log (`install.sh` prints it).
/// Mirrors `PortalConfigInstaller.install` 1:1 (backup suffix uses epoch
/// seconds instead of `%Y%m%d-%H%M%S` — no `chrono` for a timestamp).
pub fn install_config(home: &str, runner: &dyn CommandRunner) -> Result<Vec<String>, String> {
    let mut log = Vec::new();
    let executable = format!("{home}/.config/sway/bin/SwayManager");
    if !std::path::Path::new(&executable).exists() {
        return Err(format!("Executável não encontrado: {executable}"));
    }
    log.push(format!("Executável confirmado: {executable}"));

    let portal_dir = format!("{home}/.config/xdg-desktop-portal");
    let _ = std::fs::create_dir_all(&portal_dir);
    for filename in ["portals.conf", "sway-portals.conf"] {
        let path = std::path::PathBuf::from(format!("{portal_dir}/{filename}"));
        if path.exists() {
            let backup = backup_path(&path);
            if std::fs::copy(&path, &backup).is_ok() {
                log.push(format!("Backup criado: {}", backup.display()));
            }
        }
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let merged = merge_ini_section(
            &existing,
            "preferred",
            &[
                ("default", "gtk".to_string()),
                ("org.freedesktop.impl.portal.Screenshot", "wlr".to_string()),
                ("org.freedesktop.impl.portal.ScreenCast", "wlr".to_string()),
            ],
        );
        if std::fs::write(&path, merged).is_ok() {
            log.push(format!("Configuração atualizada: {}", path.display()));
        }
    }

    let wlr_path = std::path::PathBuf::from(format!("{home}/.config/xdg-desktop-portal-wlr/config"));
    if let Some(parent) = wlr_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if wlr_path.exists() {
        let backup = backup_path(&wlr_path);
        if std::fs::copy(&wlr_path, &backup).is_ok() {
            log.push(format!("Backup criado: {}", backup.display()));
        }
    }
    let existing = std::fs::read_to_string(&wlr_path).unwrap_or_default();
    let mut entries = vec![
        ("chooser_cmd", format!("{executable} portal")),
        ("chooser_type", "dmenu".to_string()),
    ];
    if !existing.lines().any(|l| l.trim().starts_with("max_fps")) {
        entries.push(("max_fps", "30".to_string()));
    }
    let merged = merge_ini_section(&existing, "screencast", &entries);
    if std::fs::write(&wlr_path, merged).is_ok() {
        log.push(format!("Configuração atualizada: {}", wlr_path.display()));
    }

    let sway_config = std::path::PathBuf::from(format!("{home}/.config/sway/config"));
    if !sway_config.exists() {
        log.push(format!("Arquivo de configuração do Sway não encontrado: {}", sway_config.display()));
    } else {
        let content = std::fs::read_to_string(&sway_config).unwrap_or_default();
        if content.contains("dbus-update-activation-environment") {
            log.push(format!("Variáveis de sessão já configuradas em: {}", sway_config.display()));
        } else {
            let backup = backup_path(&sway_config);
            if std::fs::copy(&sway_config, &backup).is_ok() {
                log.push(format!("Backup criado: {}", backup.display()));
            }
            let line = "exec dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP=sway\n";
            let appended = format!("{content}\n# SwayManager: export session variables for D-Bus services\n{line}");
            if std::fs::write(&sway_config, appended).is_ok() {
                log.push(format!("Variáveis de sessão adicionadas em: {}", sway_config.display()));
            }
        }
    }

    for service in ["xdg-desktop-portal-wlr", "xdg-desktop-portal"] {
        let out = runner.run("systemctl", &["--user", "restart", service]);
        if out.succeeded() {
            log.push(format!("Serviço reiniciado: {service}"));
        } else {
            log.push(format!("Não foi possível reiniciar {service}: {}", out.stderr.trim()));
        }
    }
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    // --- result_writer (mirrors TestPortalResultWriter) ---

    #[test]
    fn write_monitor() {
        assert_eq!(format_monitor("HDMI-A-1"), Some("HDMI-A-1\n".to_string()));
    }

    #[test]
    fn write_window() {
        assert_eq!(format_window("window-123"), Some("window-123\n".to_string()));
    }

    #[test]
    fn write_result() {
        let result = PortalResult::new(SourceType::Monitor, "eDP-1");
        assert_eq!(format_result(&result), Some("eDP-1\n".to_string()));
    }

    #[test]
    fn cancel_writes_nothing() {
        let empty = PortalResult::new(SourceType::Monitor, "");
        assert_eq!(format_result(&empty), None);
    }

    #[test]
    fn validate_identifier_rejects_multiline() {
        assert!(validate_identifier("HDMI-A-1"));
        assert!(!validate_identifier(""));
        assert!(!validate_identifier("line\nbreak"));
        assert!(!validate_identifier("line\rbreak"));
    }

    // --- outputs provider (mirrors TestSwayOutputsProvider) ---

    fn outputs_json() -> String {
        serde_json::json!([
            {
                "name": "eDP-1", "active": true, "focused": true, "primary": true,
                "make": "AU Optronics", "model": "B140HAN",
                "current_mode": {"width": 1920, "height": 1080, "refresh": 60000},
                "scale": 1.0, "transform": "normal", "rect": {"x": 0, "y": 0},
            },
            {
                "name": "HDMI-A-1", "active": false,
                "make": "Dell", "model": "U2720Q",
                "current_mode": {"width": 3840, "height": 2160, "refresh": 60000},
                "scale": 1.5, "rect": {"x": 1920, "y": 0},
            },
        ])
        .to_string()
    }

    #[test]
    fn parses_active_outputs() {
        let runner = FakeRunner::new()
            .with_success("swaymsg", &["-t", "get_outputs", "-r"], &outputs_json());
        let outputs = get_outputs(&runner).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].id, "eDP-1");
        assert_eq!(outputs[0].source_type, SourceType::Monitor);
        assert!(outputs[0].details.contains("1920 \u{d7} 1080"));
        assert!(outputs[0].details.contains("Escala: 1"));
        assert!(outputs[0].is_primary);
    }

    #[test]
    fn raises_when_swaymsg_fails() {
        let runner = FakeRunner::new().with_failure("swaymsg", &["-t", "get_outputs", "-r"], 1);
        assert!(get_outputs(&runner).is_err());
    }

    #[test]
    fn raises_when_no_active_outputs() {
        let runner = FakeRunner::new().with_success(
            "swaymsg",
            &["-t", "get_outputs", "-r"],
            r#"[{"name": "eDP-1", "active": false}]"#,
        );
        assert!(get_outputs(&runner).is_err());
    }

    // --- windows provider (mirrors TestWindowSharingProvider) ---

    #[test]
    fn unsupported_sway_returns_empty() {
        let runner = FakeRunner::new().with_success("sway", &["--version"], "sway version 1.11");
        assert!(!windows_supported(&runner));
        assert_eq!(get_windows(&runner).unwrap(), Vec::new());
    }

    #[test]
    fn supported_sway_without_lswt_returns_empty() {
        let runner = FakeRunner::new()
            .with_success("sway", &["--version"], "sway version 1.12")
            .with_failure("which", &["lswt"], 1);
        assert!(windows_supported(&runner));
        assert_eq!(get_windows(&runner).unwrap(), Vec::new());
    }

    #[test]
    fn availability_explains_unsupported_sway_version() {
        let runner = FakeRunner::new().with_success("sway", &["--version"], "sway version 1.11");
        let availability = window_availability(&runner);
        assert!(!availability.supported);
        assert!(availability.reason.unwrap().contains("Sway 1.11"));
    }

    #[test]
    fn availability_requires_lswt() {
        let runner = FakeRunner::new()
            .with_success("sway", &["--version"], "sway version 1.12")
            .with_failure("which", &["lswt"], 1);
        let availability = window_availability(&runner);
        assert!(!availability.supported);
        assert!(availability.reason.unwrap().contains("lswt"));
    }

    #[test]
    fn parses_lswt_output() {
        let toplevels = serde_json::json!([
            {"id": "toplevel-1", "app_id": "org.mozilla.firefox", "title": "Firefox", "workspace": "2", "focused": true},
        ])
        .to_string();
        let runner = FakeRunner::new()
            .with_success("sway", &["--version"], "sway version 1.12")
            .with_success("which", &["lswt"], "/usr/bin/lswt")
            .with_success("lswt", &["-j"], &toplevels);
        let windows = get_windows(&runner).unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].id, "toplevel-1");
        assert_eq!(windows[0].source_type, SourceType::Window);
        assert!(windows[0].is_focused);
    }

    #[test]
    fn parse_sway_version_for_swayfx() {
        assert_eq!(
            parse_sway_version("swayfx version 0.5.3-660c1197 (based on sway 1.11.0)"),
            Some((1, 11))
        );
    }

    #[test]
    fn parse_sway_version_with_utf8_prefix_does_not_panic() {
        // Regressão: `i += 1` atômico panica no meio de char multi-byte.
        assert_eq!(parse_sway_version("versão π: sway version 1.12"), Some((1, 12)));
    }

    #[test]
    fn quote_bare_ids_with_utf8_titles_does_not_panic() {
        // Regressão: título de janela com `π` derrubava o Portal inteiro.
        let raw = r#"[{"id": abc123, "title": "cálculo π — terminal"},
            {"identifier": def456, "title": "日本語ウィンドウ"}]"#;
        let quoted = quote_bare_ids(raw);
        assert!(quoted.contains(r#""id": "abc123""#), "{quoted}");
        assert!(quoted.contains(r#""identifier": "def456""#), "{quoted}");
        assert!(quoted.contains("cálculo π — terminal"), "{quoted}");
        assert!(quoted.contains("日本語ウィンドウ"), "{quoted}");
    }
    // --- diagnostics (mirrors TestPortalDiagnostics) ---

    fn ready_env() -> DiagnosticsEnv {
        DiagnosticsEnv {
            session_type: "wayland".to_string(),
            wayland_display: "wayland-1".to_string(),
            current_desktop: "sway".to_string(),
            sway_sock: "/run/user/1000/sway-ipc.sock".to_string(),
            session_desktop: String::new(),
        }
    }

    #[test]
    fn ready_when_all_services_active() {
        let runner = FakeRunner::new()
            .with_success("sway", &["--version"], "sway version 1.12")
            .with_success("which", &["lswt"], "/usr/bin/lswt")
            .with_success("systemctl", &["--user", "is-active", "xdg-desktop-portal"], "")
            .with_success("systemctl", &["--user", "is-active", "xdg-desktop-portal-wlr"], "")
            .with_which("pipewire", Some("/usr/bin/pipewire"))
            .with_which("pw-cli", None);
        let report = diagnose_with(&runner, &ready_env());
        assert!(report.is_wayland);
        assert_eq!(report.compositor, "sway");
        assert!(report.pipewire_available);
        assert!(report.xdg_desktop_portal_active);
        assert!(report.xdg_desktop_portal_wlr_active);
        assert!(report.session_vars_exported);
        assert!(report.is_ready());
    }

    // --- config installer (mirrors TestPortalConfigInstaller) ---

    #[test]
    fn install_writes_portal_configs_and_restarts() {
        let home = tempfile::tempdir().unwrap();
        let home = home.path().to_str().unwrap();
        std::fs::create_dir_all(format!("{home}/.config/sway/bin")).unwrap();
        std::fs::write(format!("{home}/.config/sway/bin/SwayManager"), "x").unwrap();
        std::fs::write(format!("{home}/.config/sway/config"), "set $mod Mod4\n").unwrap();
        let runner = FakeRunner::new()
            .with_success("systemctl", &["--user", "restart", "xdg-desktop-portal-wlr"], "")
            .with_success("systemctl", &["--user", "restart", "xdg-desktop-portal"], "");
        let log = install_config(home, &runner).unwrap();
        let text = log.join("\n");
        assert!(text.contains("Executável confirmado"), "{text}");
        assert!(text.contains("portals.conf"), "{text}");
        assert!(text.contains("sway-portals.conf"), "{text}");
        assert!(text.contains("xdg-desktop-portal-wlr/config"), "{text}");
        assert!(text.contains("Variáveis de sessão adicionadas"), "{text}");
        assert!(text.contains("Serviço reiniciado: xdg-desktop-portal-wlr"), "{text}");
        let wlr = std::fs::read_to_string(format!("{home}/.config/xdg-desktop-portal-wlr/config")).unwrap();
        assert!(wlr.contains("[screencast]") && wlr.contains("chooser_type = dmenu"), "{wlr}");
        assert!(wlr.contains("chooser_cmd") && wlr.contains("portal"), "{wlr}");
        let portals = std::fs::read_to_string(format!("{home}/.config/xdg-desktop-portal/portals.conf")).unwrap();
        assert!(portals.contains("org.freedesktop.impl.portal.ScreenCast = wlr"), "{portals}");
        let sway = std::fs::read_to_string(format!("{home}/.config/sway/config")).unwrap();
        assert!(sway.contains("dbus-update-activation-environment"), "{sway}");
    }

    #[test]
    fn install_fails_without_executable_and_preserves_max_fps() {
        let home = tempfile::tempdir().unwrap();
        let home = home.path().to_str().unwrap();
        let runner = FakeRunner::new();
        let err = install_config(home, &runner).unwrap_err();
        assert!(err.contains("Executável não encontrado"), "{err}");

        std::fs::create_dir_all(format!("{home}/.config/sway/bin")).unwrap();
        std::fs::write(format!("{home}/.config/sway/bin/SwayManager"), "x").unwrap();
        std::fs::create_dir_all(format!("{home}/.config/xdg-desktop-portal-wlr")).unwrap();
        std::fs::write(
            format!("{home}/.config/xdg-desktop-portal-wlr/config"),
            "[screencast]\nmax_fps = 60\n",
        )
        .unwrap();
        install_config(home, &runner).unwrap();
        let wlr = std::fs::read_to_string(format!("{home}/.config/xdg-desktop-portal-wlr/config")).unwrap();
        assert!(wlr.contains("max_fps = 60"), "preserva custom: {wlr}");
        assert!(wlr.contains("chooser_cmd"), "{wlr}");
    }
}
