//! Portal chooser + freeze overlay (slice D do porte Tauri v2).
//!
//! Espelha `src/portal/` (`controller.py`, `result_writer.py`,
//! `selection_cache.py`, dmenu-stdin) + `freeze_selection_overlay.py`
//! (`SelectionState`, `compose_selection`) sobre os repos do daemon.
//! NUNCA faz shell-out direto nem escreve em stdout: o resultado sai como
//! `Ok(String)` para quem invoca (fase 2 escreve na stdout do chooser).
//!
//! Na fase 2 cada `pub fn` ganha `#[tauri::command]`; as assinaturas já são
//! compatíveis (`Result<_, String>`, args `Deserialize`, frontend `camelCase`
//! → Rust `snake_case`). `runner` vira estado gerenciado (`State<Runner>`).

use std::path::Path;

use serde::{Deserialize, Serialize};
use sway_manager_daemon::process::CommandRunner;
use sway_manager_daemon::repos::{portal, screenshot};

/// Lado mínimo aceito (Qt: `MIN_SELECTION_SIZE = 6`).
pub const MIN_SELECTION_SIZE: i32 = 6;


/// Fonte serializada para o frontend (espelha `portal/models.PortalSource`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalSourceDto {
    pub id: String,
    /// `"monitor"` | `"window"` (espelha `PortalSourceType.value`).
    #[serde(alias = "source_type")]
    pub source_type: String,
    pub label: String,
    pub details: String,
    #[serde(alias = "is_primary")]
    pub is_primary: bool,
    #[serde(alias = "is_focused")]
    pub is_focused: bool,
    #[serde(alias = "raw_label")]
    pub raw_label: Option<String>,
}

impl PortalSourceDto {
    /// Na fase Tauri: `#[tauri::command] fn portal_sources(...)`.
    #[must_use]
    pub fn from_daemon(source: &portal::PortalSource) -> Self {
        Self {
            id: source.id.clone(),
            source_type: source.source_type.as_str().to_string(),
            label: source.label.clone(),
            details: source.details.clone(),
            is_primary: source.is_primary,
            is_focused: source.is_focused,
            raw_label: source.raw_label.clone(),
        }
    }
}

/// Seleção serializada (espelha `portal/models.PortalResult`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalSelection {
    #[serde(alias = "source_type")]
    pub source_type: String,
    pub id: String,
    #[serde(alias = "raw_label")]
    pub raw_label: Option<String>,
}

impl PortalSelection {
    #[must_use]
    pub fn from_daemon(result: &portal::PortalResult) -> Self {
        Self {
            source_type: result.source_type.as_str().to_string(),
            id: result.id.clone(),
            raw_label: result.raw_label.clone(),
        }
    }

    #[must_use]
    pub fn to_daemon(&self) -> portal::PortalResult {
        portal::PortalResult {
            source_type: portal::SourceType::parse(&self.source_type),
            id: self.id.clone(),
            raw_label: self.raw_label.clone(),
        }
    }

    /// Linha emitida ao backend (`PortalResult.__str__`).
    #[must_use]
    pub fn line(&self) -> &str {
        self.raw_label.as_deref().unwrap_or(&self.id)
    }
}

/// Fontes descobertas (espelha `PortalController.run`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalSources {
    pub monitors: Vec<PortalSourceDto>,
    pub windows: Vec<PortalSourceDto>,
    /// Motivo da indisponibilidade das janelas (`_load_windows`), se houver.
    pub window_reason: Option<String>,
}

/// Retângulo em coordenadas lógicas globais (espelha `QRect`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.w <= 0 || self.h <= 0
    }

    /// Interseção (espelha `QRect.intersected`).
    #[must_use]
    pub fn intersect(self, other: Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.w).min(other.x + other.w);
        let bottom = (self.y + self.h).min(other.y + other.h);
        Rect {
            x,
            y,
            w: right - x,
            h: bottom - y,
        }
    }
}

/// Tela congelada (espelha `FrozenScreen`; a imagem vive no frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrozenScreenDto {
    pub name: String,
    pub geometry: Rect,
    pub scale_x: f64,
    pub scale_y: f64,
    /// Caminho do `grim -o` para o `<img>`/canvas do frontend.
    pub image: Option<String>,
}

/// Um blit do plano de composição (origem física → destino no canvas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlitOp {
    pub screen: usize,
    pub src: Rect,
    pub dst: Rect,
}

/// Plano de composição (espelha `compose_selection` sem os pixels Qt).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub width: u32,
    pub height: u32,
    pub ops: Vec<BlitOp>,
}

/// Máquina de um arrasto (espelha `SelectionState`, coordenadas lógicas).
#[derive(Debug, Default, Clone, Copy)]
pub struct SelectionState {
    start: Option<(i32, i32)>,
    current: Option<(i32, i32)>,
}

impl SelectionState {
    /// Na fase Tauri: estado vive no frontend; isto valida `freeze_compose`.
    pub fn begin(&mut self, x: i32, y: i32) {
        self.start = Some((x, y));
        self.current = Some((x, y));
    }

    pub fn update(&mut self, x: i32, y: i32) {
        if self.start.is_some() {
            self.current = Some((x, y));
        }
    }

    /// Retângulo normalizado inclusivo (`QRect`: `abs + 1`).
    #[must_use]
    pub fn rectangle(&self) -> Option<Rect> {
        let (sx, sy) = self.start?;
        let (cx, cy) = self.current?;
        Some(Rect {
            x: sx.min(cx),
            y: sy.min(cy),
            w: (cx - sx).abs() + 1,
            h: (cy - sy).abs() + 1,
        })
    }

    /// `None` quando menor que o mínimo (o Qt cancela a seleção).
    pub fn finish(&mut self, x: i32, y: i32) -> Option<Rect> {
        self.update(x, y);
        let rect = self.rectangle()?;
        if rect.w < MIN_SELECTION_SIZE || rect.h < MIN_SELECTION_SIZE {
            return None;
        }
        Some(rect)
    }
}


/// Parseia rótulos dmenu do xdpw (espelha `_parse_dmenu_sources`).
///
/// 0.7.x emite um nome de output por linha; versões novas emitem
/// `Monitor: <nome> <descrição>` / `Window: <título> (<id>)`.
/// O texto exato (`raw_label`) é preservado para o exact-match do backend.
#[must_use]
pub fn parse_dmenu_sources(content: &str) -> (Vec<portal::PortalSource>, Vec<portal::PortalSource>) {
    let mut monitors = Vec::new();
    let mut windows = Vec::new();
    for line in content.lines() {
        let stripped = line.trim_end();
        if stripped.trim().is_empty() {
            continue;
        }
        if let Some(rest) = stripped.strip_prefix("Monitor: ") {
            let mut parts = rest.splitn(2, char::is_whitespace);
            let name = parts.next().unwrap_or("");
            let description = parts.next().unwrap_or("").trim().to_string();
            if !name.is_empty() {
                monitors.push(portal::PortalSource {
                    id: name.to_string(),
                    source_type: portal::SourceType::Monitor,
                    label: name.to_string(),
                    details: description,
                    is_primary: false,
                    is_focused: false,
                    raw_label: Some(stripped.to_string()),
                });
            }
        } else if let Some(rest) = stripped.strip_prefix("Window: ") {
            let (identifier, title) = split_window_label(rest);
            if !identifier.is_empty() {
                let label = if title.is_empty() {
                    identifier.clone()
                } else {
                    title
                };
                windows.push(portal::PortalSource {
                    id: identifier,
                    source_type: portal::SourceType::Window,
                    label,
                    details: String::new(),
                    is_primary: false,
                    is_focused: false,
                    raw_label: Some(stripped.to_string()),
                });
            }
        } else {
            monitors.push(portal::PortalSource {
                id: stripped.to_string(),
                source_type: portal::SourceType::Monitor,
                label: stripped.to_string(),
                details: String::new(),
                is_primary: false,
                is_focused: false,
                raw_label: Some(stripped.to_string()),
            });
        }
    }
    (monitors, windows)
}

/// Divide `"<título> (<id-hex>)"` (espelha o `re.search` do controller).
fn split_window_label(rest: &str) -> (String, String) {
    let trimmed = rest.trim_end();
    if let Some(close) = trimmed.strip_suffix(')')
        && let Some(open) = close.rfind('(')
    {
        let identifier = close[open + 1..].to_string();
        if !identifier.is_empty() && identifier.chars().all(|c| c.is_ascii_hexdigit()) {
            return (identifier, close[..open].trim().to_string());
        }
    }
    let identifier = trimmed.to_string();
    (identifier.clone(), identifier)
}

/// Descobre fontes: dmenu-stdin OU swaymsg+lswt (sempre interativo).
///
/// Na fase Tauri: `#[tauri::command] fn portal_sources(stdin_text: Option<String>)`.
pub fn portal_sources(
    runner: &dyn CommandRunner,
    stdin_text: Option<String>,
) -> Result<PortalSources, String> {
    let piped = stdin_text.map(|text| text.trim().to_string()).filter(|text| !text.is_empty());
    let (monitors, windows, window_reason) = if let Some(content) = piped {
        let (monitors, windows) = parse_dmenu_sources(&content);
        (monitors, windows, None)
    } else {
        let monitors = portal::get_outputs(runner)?;
        let availability = portal::window_availability(runner);
        let (windows, reason) = if !availability.supported {
            (Vec::new(), availability.reason)
        } else {
            match portal::list_windows(runner) {
                Ok(found) if found.is_empty() => (
                    found,
                    Some("Nenhuma janela compartilhável foi encontrada nesta sessão.".to_string()),
                ),
                Ok(found) => (found, None),
                Err(_) => (
                    Vec::new(),
                    Some("Não foi possível consultar as janelas compartilháveis.".to_string()),
                ),
            }
        };
        (monitors, windows, reason)
    };
    Ok(PortalSources {
        monitors: monitors.iter().map(PortalSourceDto::from_daemon).collect(),
        windows: windows.iter().map(PortalSourceDto::from_daemon).collect(),
        window_reason,
    })
}

/// Valida + formata a seleção; NUNCA escreve em stdout.
///
/// `result_json` é `{"source_type","id","raw_label?"}`; retorna a linha exata
/// que o chooser emite ao xdpw. `None`-cancelamento = [`portal_cancel`].
///
/// Na fase Tauri: `#[tauri::command] fn portal_select(result_json: String)`.
pub fn portal_select(result_json: &str) -> Result<String, String> {
    let selection: PortalSelection =
        serde_json::from_str(result_json).map_err(|err| format!("resultado inválido: {err}"))?;
    if !portal::validate_identifier(&selection.id) {
        return Err("identificador inválido para o portal".to_string());
    }
    if selection.raw_label.as_deref().is_some_and(|raw| !portal::validate_identifier(raw)) {
        return Err("identificador inválido para o portal".to_string());
    }
    if selection.line().is_empty() {
        return Err("resultado vazio: nada a compartilhar".to_string());
    }
    let result = selection.to_daemon();
    if portal::format_result(&result).is_none() {
        return Err("resultado vazio: nada a compartilhar".to_string());
    }
    Ok(result.line().to_string())
}

/// Cancelamento: emite nada em stdout, exit 0 (espelha `cancel()`).
#[must_use]
pub fn portal_cancel() -> Option<String> {
    None
}

/// Congela um output (`grim -o`, espelha `capture_output`).
///
/// Na fase Tauri: `#[tauri::command] fn freeze_capture(output: String, dst: String)`.
pub fn freeze_capture(
    runner: &dyn CommandRunner,
    output: &str,
    dst: &str,
) -> Result<String, String> {
    if output.trim().is_empty() || dst.trim().is_empty() {
        return Err("output ou destino inválido para o freeze".to_string());
    }
    let is_file = |path: &str| Path::new(path).is_file();
    if screenshot::capture_output(runner, output, dst, &is_file) {
        Ok(dst.to_string())
    } else {
        Err(format!("falha ao congelar o output {output}"))
    }
}

const B64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 4);
    for chunk in bytes.chunks(3) {
        let mut buf: u32 = 0;
        for (i, &b) in chunk.iter().enumerate() {
            buf |= (b as u32) << (16 - 8 * i);
        }
        out.push(B64_TABLE[(buf >> 18) as usize & 63] as char);
        out.push(B64_TABLE[(buf >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { B64_TABLE[(buf >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64_TABLE[buf as usize & 63] as char } else { '=' });
    }
    out
}

/// PNG do freeze como `data:image/png;base64,...` (o WebKit bloqueia
/// `file://` e o `asset://` exige scope; data-URL sempre renderiza).
pub fn freeze_image_data(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("falha ao ler freeze: {e}"))?;
    Ok(format!("data:image/png;base64,{}", base64_encode(&bytes)))
}

/// Compõe a seleção lógica no maior scale participante.
///
/// Espelha `compose_selection` (interseções, `ceil`, `round`, gap
/// transparente = ausência de op); os pixels são blitados no frontend
/// (canvas) a partir do [`Composition`] retornado.
///
/// Na fase Tauri: `#[tauri::command] fn freeze_compose(selection: Rect, screens: Vec<FrozenScreenDto>, output: String)`.
#[must_use]
pub fn freeze_compose(selection: Rect, screens: &[FrozenScreenDto]) -> Option<Composition> {
    if selection.is_empty() {
        return None;
    }
    let mut hits: Vec<(usize, Rect)> = Vec::new();
    for (index, screen) in screens.iter().enumerate() {
        let hit = selection.intersect(screen.geometry);
        if !hit.is_empty() {
            hits.push((index, hit));
        }
    }
    if hits.is_empty() {
        return None;
    }
    let output_scale = hits
        .iter()
        .map(|(index, _)| screens[*index].scale_x.max(screens[*index].scale_y))
        .fold(0.0_f64, f64::max);
    if output_scale <= 0.0 {
        return None;
    }
    let width = (f64::from(selection.w) * output_scale).ceil();
    let height = (f64::from(selection.h) * output_scale).ceil();
    let mut ops = Vec::with_capacity(hits.len());
    for (index, hit) in hits {
        let screen = &screens[index];
        let local_x = hit.x - screen.geometry.x;
        let local_y = hit.y - screen.geometry.y;
        let src = Rect {
            x: (f64::from(local_x) * screen.scale_x).round() as i32,
            y: (f64::from(local_y) * screen.scale_y).round() as i32,
            w: ((f64::from(local_x + hit.w) * screen.scale_x).round()
                - (f64::from(local_x) * screen.scale_x).round()) as i32,
            h: ((f64::from(local_y + hit.h) * screen.scale_y).round()
                - (f64::from(local_y) * screen.scale_y).round()) as i32,
        };
        let rel_x = hit.x - selection.x;
        let rel_y = hit.y - selection.y;
        let dst = Rect {
            x: (f64::from(rel_x) * output_scale).round() as i32,
            y: (f64::from(rel_y) * output_scale).round() as i32,
            w: ((f64::from(rel_x + hit.w) * output_scale).round()
                - (f64::from(rel_x) * output_scale).round()) as i32,
            h: ((f64::from(rel_y + hit.h) * output_scale).round()
                - (f64::from(rel_y) * output_scale).round()) as i32,
        };
        ops.push(BlitOp { screen: index, src, dst });
    }
    Some(Composition {
        width: width as u32,
        height: height as u32,
        ops,
    })
}

/// Telas ativas com geometria lógica + scale (espelha o parse de
/// `SwayOutputsProvider`; o frontend usa para `boot()` e `screens`).
#[must_use]
pub fn freeze_screens(runner: &dyn CommandRunner) -> Vec<FrozenScreenDto> {
    let out = runner.run("swaymsg", &["-t", "get_outputs", "-r"]);
    if !out.succeeded() {
        return Vec::new();
    }
    let Ok(outputs) = serde_json::from_str::<Vec<serde_json::Value>>(&out.stdout) else {
        return Vec::new();
    };
    outputs
        .iter()
        .filter(|o| o.get("active").and_then(serde_json::Value::as_bool).unwrap_or(false))
        .filter_map(|o| {
            let name = o.get("name")?.as_str()?.to_string();
            let rect = o.get("rect")?;
            let scale = o.get("scale").and_then(serde_json::Value::as_f64).unwrap_or(1.0);
            Some(FrozenScreenDto {
                name,
                geometry: Rect {
                    x: rect.get("x")?.as_i64()? as i32,
                    y: rect.get("y")?.as_i64()? as i32,
                    w: rect.get("width")?.as_i64()? as i32,
                    h: rect.get("height")?.as_i64()? as i32,
                },
                scale_x: scale,
                scale_y: scale,
                image: None,
            })
        })
        .collect()
}

/// Timestamp `YYYY-MM-DD_HH:MM:SS` em hora LOCAL (espelha
/// `TakeScreenshotUseCase`: `datetime.now()`, não UTC).
#[must_use]
pub fn screenshot_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // `libc::localtime_r` respeita TZ (paridade `datetime.now()`); sem ele
    // o nome do arquivo sairia em UTC (3h adiantado no Brasil).
    let mut out = std::mem::MaybeUninit::<libc::tm>::uninit();
    // SAFETY: `localtime_r` com `tm` válido é thread-safe; `now` é epoch válido.
    let tm = unsafe {
        libc::localtime_r(&now as *const i64 as *const libc::time_t, out.as_mut_ptr());
        out.assume_init()
    };
    format!(
        "{:04}-{:02}-{:02}_{:02}:{:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

/// Decodifica base64 padrão (sem dependências; só o que `toDataURL` emite).
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits = 0;
    for byte in input.bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => return None,
        } as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// Salva o PNG do canvas (`data:image/png;base64,...`), copia para o
/// clipboard (`wl-copy`) e notifica — paridade `publish_screenshot`.
/// Retorna o caminho salvo ou a mensagem de erro.
pub fn screenshot_save_png(
    runner: &dyn CommandRunner,
    data_url: &str,
    folder: &str,
    home: &str,
) -> Result<String, String> {
    let payload = data_url.strip_prefix("data:image/png;base64,").ok_or_else(|| "data-url inválida".to_string())?;
    let bytes = base64_decode(payload).ok_or_else(|| "base64 inválido".to_string())?;
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("conteúdo não é PNG".to_string());
    }
    let folder = if folder.is_empty() {
        let saved = sway_manager_daemon::repos::app_config::get_setting(
            &sway_manager_daemon::repos::app_config::default_config_path(home),
            "screenshot_folder",
        )
        .and_then(|v| v.as_str().map(str::to_string));
        screenshot::resolve_screenshot_folder(runner, saved.as_deref(), home, &|p| {
            std::path::Path::new(p).is_dir()
        })
    } else {
        folder.to_string()
    };
    let _ = std::fs::create_dir_all(&folder);
    let path = screenshot::screenshot_filename(&folder, &screenshot_timestamp());
    std::fs::write(&path, &bytes).map_err(|e| format!("falha ao salvar: {e}"))?;
    screenshot::publish_screenshot(runner, screenshot::ScreenshotMode::Area, &path, &|p| {
        std::path::Path::new(p).is_file()
    });
    Ok(path)
}

/// Captura tela cheia sem janela (`grim <dst>` + publish).
pub fn screenshot_full(runner: &dyn CommandRunner, home: &str) -> Result<String, String> {
    let saved = sway_manager_daemon::repos::app_config::get_setting(
        &sway_manager_daemon::repos::app_config::default_config_path(home),
        "screenshot_folder",
    )
    .and_then(|v| v.as_str().map(str::to_string));
    let folder = screenshot::resolve_screenshot_folder(runner, saved.as_deref(), home, &|p| {
        std::path::Path::new(p).is_dir()
    });
    let _ = std::fs::create_dir_all(&folder);
    let path = screenshot::screenshot_filename(&folder, &screenshot_timestamp());
    let is_file = |p: &str| std::path::Path::new(p).is_file();
    if !screenshot::capture_full(runner, &path, &is_file) {
        return Err("falha ao capturar a tela".to_string());
    }
    screenshot::publish_screenshot(runner, screenshot::ScreenshotMode::Full, &path, &is_file);
    Ok(path)
}

/// Captura a janela focada sem overlay (`swaymsg get_tree` + `grim -g`).
pub fn screenshot_window(runner: &dyn CommandRunner, home: &str) -> Result<String, String> {
    let saved = sway_manager_daemon::repos::app_config::get_setting(
        &sway_manager_daemon::repos::app_config::default_config_path(home),
        "screenshot_folder",
    )
    .and_then(|v| v.as_str().map(str::to_string));
    let folder = screenshot::resolve_screenshot_folder(runner, saved.as_deref(), home, &|p| {
        std::path::Path::new(p).is_dir()
    });
    let _ = std::fs::create_dir_all(&folder);
    let path = screenshot::screenshot_filename(&folder, &screenshot_timestamp());
    let is_file = |p: &str| std::path::Path::new(p).is_file();
    if !screenshot::capture_window(runner, &path, &is_file) {
        return Err("nenhuma janela focada para capturar".to_string());
    }
    screenshot::publish_screenshot(runner, screenshot::ScreenshotMode::Window, &path, &is_file);
    Ok(path)
}

/// Cancelamento do overlay: noop que fecha a janela (o frontend chama
/// `window.close()` após o invoke; sem este comando o invoke rejeita).
#[must_use]
pub fn freeze_cancel() -> Option<String> {
    None
}

/// Compõe a seleção em pixels NATIVOS e salva (substitui o canvas).
///
/// Paridade `compose_selection`: interseção por output, coordenadas físicas
/// (`× scale`, `round`), gaps transparentes. Diferença do Qt: o Qt blitava
/// `QImage` em memória; aqui decodificamos os PNGs do `grim -o` com a crate
/// `image` (só feature `png`). Cross-output funciona: cada tela contribui
/// seus pixels na posição certa do canvas final.
pub fn screenshot_compose_save(
    runner: &dyn CommandRunner,
    selection: Rect,
    screens: &[FrozenScreenDto],
    folder: &str,
    home: &str,
) -> Result<String, String> {
    if selection.is_empty() {
        return Err("seleção vazia".to_string());
    }
    let plan = freeze_compose(selection, screens).ok_or_else(|| "seleção fora das telas".to_string())?;
    if plan.ops.is_empty() {
        return Err("seleção fora das telas".to_string());
    }
    let mut canvas = image::RgbaImage::new(plan.width, plan.height);
    for op in &plan.ops {
        let Some(screen) = screens.get(op.screen) else { continue };
        let Some(path) = screen.image.as_deref().filter(|p| !p.is_empty()) else {
            continue;
        };
        let Ok(img) = image::open(path).map(|i| i.to_rgba8()) else {
            continue;
        };
        let (iw, ih) = (img.width() as i32, img.height() as i32);
        // Clampa o src aos pixels reais (grim pode variar 1px por rounding).
        let sx = op.src.x.clamp(0, iw);
        let sy = op.src.y.clamp(0, ih);
        let sw = (op.src.w).min(iw - sx).max(0) as u32;
        let sh = (op.src.h).min(ih - sy).max(0) as u32;
        if sw == 0 || sh == 0 {
            continue;
        }
        let view = image::imageops::crop_imm(&img, sx as u32, sy as u32, sw, sh).to_image();
        // Paridade Qt (`drawImage(src, dst)`): o QPainter ESCALAVA src→dst.
        // Sem isso, telas com scale menor que o output_scale saem com
        // metade da altura/largura. Nearest = nítido, sem blur.
        let dx = op.dst.x.max(0) as u32;
        let dy = op.dst.y.max(0) as u32;
        let dw = (op.dst.w.max(0) as u32).min(plan.width.saturating_sub(dx));
        let dh = (op.dst.h.max(0) as u32).min(plan.height.saturating_sub(dy));
        if dw == 0 || dh == 0 {
            continue;
        }
        let view = if view.width() == dw && view.height() == dh {
            view
        } else {
            image::imageops::resize(&view, dw, dh, image::imageops::FilterType::Nearest)
        };
        image::imageops::overlay(&mut canvas, &view, dx as i64, dy as i64);
    }
    let folder = if folder.is_empty() {
        let saved = sway_manager_daemon::repos::app_config::get_setting(
            &sway_manager_daemon::repos::app_config::default_config_path(home),
            "screenshot_folder",
        )
        .and_then(|v| v.as_str().map(str::to_string));
        screenshot::resolve_screenshot_folder(runner, saved.as_deref(), home, &|p| {
            std::path::Path::new(p).is_dir()
        })
    } else {
        folder.to_string()
    };
    let _ = std::fs::create_dir_all(&folder);
    let path = screenshot::screenshot_filename(&folder, &screenshot_timestamp());
    canvas
        .save_with_format(&path, image::ImageFormat::Png)
        .map_err(|e| format!("falha ao salvar: {e}"))?;
    screenshot::publish_screenshot(runner, screenshot::ScreenshotMode::Area, &path, &|p| {
        std::path::Path::new(p).is_file()
    });
    Ok(path)
}
