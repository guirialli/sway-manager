//! Diálogo de autenticação polkit — modelo puro + framing do transporte GUI↔BE.
//!
//! Contrato vinculante (`specs/domains/polkit.md` §5):
//! - argv: `SwayManagerGUI polkit --action-id <id> --message <m>
//!   --identities <csv> --pipe <fd3> --events <fd4>`, onde cada item do csv é
//!   `unix-user:<username>` (único kind emitido pelo BE hoje).
//! - GUI→BE (fd 3): `{"password":"…"}` | `{"cancel":true}` (linhas `\n`).
//!   O transporte não carrega identity: o BE aplica a senha à PRIMEIRA
//!   identity da lista; o `<select>` reflete a lista (display sem o prefixo).
//! - BE→GUI (fd 4): `{"prompt":"…","echo":bool}` | `{"error":"…"}`
//!   | `{"info":"…"}` | `{"completed":bool}`.
//! - Senha com `\n` nunca é enviada (invariante do protocolo PAM); o cookie
//!   NUNCA cruza para a GUI (não está no argv por contrato).

use serde::{Deserialize, Serialize};

/// Uso impresso quando falta/invalida flag (fail-secure: sem janela silenciosa).
pub const POLKIT_USAGE: &str = "Uso: SwayManagerGUI polkit --action-id <id> --message <msg> --identities <csv> --pipe <fd> --events <fd>";

/// Estado do diálogo exposto ao frontend (`polkit_state`, first paint).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolkitDialogState {
    pub action_id: String,
    pub message: String,
    pub identities: Vec<String>,
    pub selected: usize,
    pub echo: bool,
    pub prompt: String,
    pub info: Option<String>,
    pub error: Option<String>,
}

/// Argv já validado (tudo após `SwayManagerGUI polkit`).
#[derive(Debug, Clone, PartialEq)]
pub struct PolkitLaunch {
    pub action_id: String,
    pub message: String,
    pub identities: Vec<String>,
    pub pipe_fd: i32,
    pub events_fd: i32,
}

/// Evento BE→GUI (uma linha JSON no fd `--events`).
#[derive(Debug, Clone, PartialEq)]
pub enum GuiEvent {
    Prompt { prompt: String, echo: bool },
    Error(String),
    Info(String),
    Completed(bool),
}

/// Nome exibido no `<select>` para uma identity do argv: `unix-user:<nome>`
/// vira `<nome>`; qualquer outro formato é exibido verbatim (sem inventar
/// parse — o BE hoje só emite `unix-user`, fallback preserva desconhecidos).
pub fn identity_display(identity: &str) -> &str {
    identity.strip_prefix("unix-user:").unwrap_or(identity)
}

impl PolkitDialogState {
    /// First paint, antes do primeiro evento do backend.
    pub fn initial(launch: &PolkitLaunch) -> Self {
        Self {
            action_id: launch.action_id.clone(),
            message: launch.message.clone(),
            identities: launch.identities.clone(),
            selected: 0,
            echo: false,
            prompt: launch.message.clone(),
            info: None,
            error: None,
        }
    }

    /// Aplica um evento no estado (o frontend também o recebe via `polkit-event`).
    pub fn apply(&mut self, event: &GuiEvent) {
        match event {
            GuiEvent::Prompt { prompt, echo } => {
                self.prompt = prompt.clone();
                self.echo = *echo;
                self.error = None;
            }
            GuiEvent::Error(message) => self.error = Some(message.clone()),
            GuiEvent::Info(message) => self.info = Some(message.clone()),
            GuiEvent::Completed(_) => {}
        }
    }
}

impl GuiEvent {
    /// Payload re-emitido ao frontend (`polkit-event`, mesmo formato do fio).
    pub fn payload(&self) -> serde_json::Value {
        match self {
            GuiEvent::Prompt { prompt, echo } => serde_json::json!({ "prompt": prompt, "echo": echo }),
            GuiEvent::Error(message) => serde_json::json!({ "error": message }),
            GuiEvent::Info(message) => serde_json::json!({ "info": message }),
            GuiEvent::Completed(done) => serde_json::json!({ "completed": done }),
        }
    }
}

/// Valida o argv do `SwayManagerGUI polkit`; flag faltante/inválida => `Err(usage)`.
pub fn parse_polkit_args(args: &[String]) -> Result<PolkitLaunch, String> {
    let mut action_id: Option<String> = None;
    let mut message: Option<String> = None;
    let mut identities: Option<Vec<String>> = None;
    let mut pipe_fd: Option<i32> = None;
    let mut events_fd: Option<i32> = None;

    let mut i = 0;
    while i < args.len() {
        // Aceita `--flag valor` e `--flag=valor`.
        let (flag, inline) = match args[i].split_once('=') {
            Some((flag, value)) => (flag, Some(value)),
            None => (args[i].as_str(), None),
        };
        let mut value = || -> Result<String, String> {
            if let Some(inline) = inline {
                return Ok(inline.to_string());
            }
            i += 1;
            args.get(i).cloned().ok_or_else(|| POLKIT_USAGE.to_string())
        };
        match flag {
            "--action-id" => action_id = Some(value()?),
            "--message" => message = Some(value()?),
            "--identities" => {
                identities = Some(
                    value()?
                        .split(',')
                        .map(|entry| entry.trim().to_string())
                        .filter(|entry| !entry.is_empty())
                        .collect(),
                );
            }
            "--pipe" => {
                pipe_fd = Some(
                    value()?
                        .parse::<i32>()
                        .ok()
                        .filter(|fd| *fd >= 0)
                        .ok_or_else(|| POLKIT_USAGE.to_string())?,
                );
            }
            "--events" => {
                events_fd = Some(
                    value()?
                        .parse::<i32>()
                        .ok()
                        .filter(|fd| *fd >= 0)
                        .ok_or_else(|| POLKIT_USAGE.to_string())?,
                );
            }
            _ => return Err(POLKIT_USAGE.to_string()),
        }
        i += 1;
    }

    match (action_id, message, identities, pipe_fd, events_fd) {
        (Some(action_id), Some(message), Some(identities), Some(pipe_fd), Some(events_fd)) => {
            Ok(PolkitLaunch {
                action_id,
                message,
                identities,
                pipe_fd,
                events_fd,
            })
        }
        _ => Err(POLKIT_USAGE.to_string()),
    }
}

/// Mapeia uma linha BE→GUI em evento; malformada => `None` (ignorada, nunca completa).
pub fn gui_event_from_json(line: &str) -> Option<GuiEvent> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let event = value.as_object()?;
    if let Some(prompt) = event.get("prompt") {
        // `echo` ausente => `false` (fail-secure: sem eco por padrão).
        let echo = event.get("echo").and_then(|echo| echo.as_bool()).unwrap_or(false);
        return Some(GuiEvent::Prompt {
            prompt: prompt.as_str()?.to_string(),
            echo,
        });
    }
    if let Some(message) = event.get("error") {
        return Some(GuiEvent::Error(message.as_str()?.to_string()));
    }
    if let Some(message) = event.get("info") {
        return Some(GuiEvent::Info(message.as_str()?.to_string()));
    }
    if let Some(done) = event.get("completed") {
        return Some(GuiEvent::Completed(done.as_bool()?));
    }
    None
}

/// Linha GUI→BE com o segredo (framing exato do contrato, `\n` final).
/// Senha com `\n` => `None` (invariante do protocolo PAM: nunca enviada).
pub fn submit_line(password: &str) -> Option<String> {
    if password.contains('\n') {
        return None;
    }
    let value = serde_json::json!({ "password": password });
    // `to_string` de `Value` é infalível.
    Some(format!("{}\n", serde_json::to_string(&value).expect("serializar Value nunca falha")))
}

/// Linha GUI→BE de aborto (framing exato do contrato, `\n` final).
pub fn cancel_line() -> String {
    format!("{}\n", serde_json::json!({ "cancel": true }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(flags: &[&str]) -> Vec<String> {
        flags.iter().map(|flag| flag.to_string()).collect()
    }

    #[test]
    fn parses_full_argv() {
        let launch = parse_polkit_args(&argv(&[
            "--action-id",
            "org.freedesktop.policykit.exec",
            "--message",
            "É necessária autenticação",
            "--identities",
            "unix-user:alice,unix-user:bob",
            "--pipe",
            "3",
            "--events",
            "4",
        ]))
        .unwrap();
        assert_eq!(launch.action_id, "org.freedesktop.policykit.exec");
        assert_eq!(
            launch.identities,
            vec!["unix-user:alice".to_string(), "unix-user:bob".to_string()]
        );
        assert_eq!((launch.pipe_fd, launch.events_fd), (3, 4));
        // Display do `<select>`: sem o prefixo `unix-user:`.
        assert_eq!(identity_display("unix-user:alice"), "alice");
        // Formato desconhecido: verbatim (BE hoje só emite `unix-user`).
        assert_eq!(identity_display("outro:1"), "outro:1");
    }

    #[test]
    fn missing_or_invalid_flag_returns_usage() {
        for flags in [
            vec!["--action-id", "a"],
            vec!["--action-id"],
            vec![
                "--action-id",
                "a",
                "--message",
                "m",
                "--identities",
                "u",
                "--pipe",
                "x",
                "--events",
                "4",
            ],
            vec!["--bogus", "1"],
        ] {
            assert_eq!(parse_polkit_args(&argv(&flags)), Err(POLKIT_USAGE.to_string()));
        }
    }

    #[test]
    fn maps_wire_events_and_ignores_malformed() {
        assert_eq!(
            gui_event_from_json(r#"{"prompt":"Senha:","echo":false}"#),
            Some(GuiEvent::Prompt {
                prompt: "Senha:".to_string(),
                echo: false
            })
        );
        assert_eq!(
            gui_event_from_json(r#"{"completed":true}"#),
            Some(GuiEvent::Completed(true))
        );
        for line in ["", "lixo", r#"{"completed":tru"#, r#"{"x":1}"#, r#"{"prompt":1}"#] {
            assert_eq!(gui_event_from_json(line), None, "{line:?}");
        }
    }

    #[test]
    fn framing_is_byte_exact() {
        assert_eq!(submit_line("s3cr3t").as_deref(), Some("{\"password\":\"s3cr3t\"}\n"));
        assert_eq!(cancel_line(), "{\"cancel\":true}\n");
        // Invariante PAM: senha com `\n` nunca é enviada.
        assert_eq!(submit_line("a\nb"), None);
    }
}
