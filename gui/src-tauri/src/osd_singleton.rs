//! OSD singleton: uma janela por kind, atualizada in-place.
//!
//! Cada tecla (`XF86AudioRaiseVolume` etc.) spawnava um processo OSD novo —
//! flicker + boot WebKit repetido. Agora a primeira instância vira servidora
//! num socket (`sway-manager-osd-<kind>-<uid>.sock`); invocações seguintes
//! enviam `kind action` e saem, e a servidora executa a ação e emite
//! `osd-update` para o frontend (que atualiza a barra e reseta o timer).

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
/// Resolve o kind OSD de `(cmd, sub)`: `osd volume` → volume,
/// `osd brilho`/`brilho up` → brightness. `None` = não-OSD.
pub fn osd_kind(cmd: &str, sub: Option<&str>) -> Option<&'static str> {
    match (cmd, sub) {
        ("osd", Some("volume")) => Some("volume"),
        ("osd", Some("brilho" | "brightness")) => Some("brightness"),
        ("brilho" | "brightness", Some("up" | "down")) => Some("brightness"),
        _ => None,
    }
}

/// Caminho do socket singleton por kind.
pub fn socket_path(kind: &str, uid: u32, runtime_dir: Option<&str>) -> PathBuf {
    let dir = runtime_dir
        .map(str::to_string)
        .or_else(|| std::env::var("XDG_RUNTIME_DIR").ok())
        .unwrap_or_else(|| "/tmp".to_string());
    PathBuf::from(dir).join(format!("sway-manager-osd-{kind}-{uid}.sock"))
}

/// Tenta notificar a instância viva: conecta, envia `action\n`, sai 0.
///
/// `Ok(true)` = notificado (chamador deve sair); `Ok(false)` = sem
/// servidor (chamador vira servidor); socket stale é removido.
pub fn try_notify(path: &std::path::Path, action: &str) -> std::io::Result<bool> {
    match UnixStream::connect(path) {
        Ok(mut stream) => {
            stream.write_all(action.as_bytes())?;
            stream.write_all(b"\n")?;
            Ok(true)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
            // Servidor morto sem cleanup: remove o stale e vira servidor.
            let _ = std::fs::remove_file(path);
            Ok(false)
        }
        Err(err) => Err(err),
    }
}

/// Vira servidor: binda (limpando stale) e retorna o listener.
pub fn bind_server(path: &std::path::Path) -> std::io::Result<UnixListener> {
    let _ = std::fs::remove_file(path);
    UnixListener::bind(path)
}

/// Lê uma ação (`up\n`) de uma conexão aceita.
pub fn read_action(mut stream: UnixStream) -> Option<String> {
    let mut buf = String::new();
    stream.read_to_string(&mut buf).ok()?;
    let action = buf.trim().to_string();
    (!action.is_empty()).then_some(action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_resolves_osd_routes() {
        assert_eq!(osd_kind("osd", Some("volume")), Some("volume"));
        assert_eq!(osd_kind("osd", Some("brilho")), Some("brightness"));
        assert_eq!(osd_kind("osd", Some("brightness")), Some("brightness"));
        assert_eq!(osd_kind("brilho", Some("up")), Some("brightness"));
        assert_eq!(osd_kind("brilho", Some("down")), Some("brightness"));
        assert_eq!(osd_kind("brilho", None), None);
        assert_eq!(osd_kind("osd", Some("volume")), Some("volume"));
        assert_eq!(osd_kind("osd", None), None);
        assert_eq!(osd_kind("settings", None), None);
        assert_eq!(osd_kind("wallpaper", None), None);
    }

    #[test]
    fn socket_path_includes_kind_uid_and_dir() {
        let path = socket_path("volume", 1000, Some("/run/user/1000"));
        assert_eq!(
            path.to_str().unwrap(),
            "/run/user/1000/sway-manager-osd-volume-1000.sock"
        );
    }

    #[test]
    fn notify_delivers_action_to_server() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("osd.sock");
        assert!(!try_notify(&path, "up").unwrap(), "sem servidor = false");
        let listener = bind_server(&path).unwrap();
        assert!(try_notify(&path, "up").unwrap(), "com servidor = true");
        let (stream, _) = listener.accept().unwrap();
        assert_eq!(read_action(stream).as_deref(), Some("up"));
    }

    #[test]
    fn stale_socket_is_reclaimed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stale.sock");
        // Bind + drop = socket órfão no fs (connect recusa).
        drop(bind_server(&path).unwrap());
        assert!(!try_notify(&path, "up").unwrap(), "stale = false");
        assert!(!path.exists(), "stale removido");
        let _listener = bind_server(&path).unwrap();
        assert!(try_notify(&path, "down").unwrap());
    }
}
