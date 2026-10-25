//! Testes do diálogo polkit — oráculo: `specs/domains/polkit.md` §5.

use sway_manager_gui::domains::system::polkit_dialog::{
    GuiEvent, POLKIT_USAGE, cancel_line, gui_event_from_json, parse_polkit_args, submit_line,
};

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
    assert_eq!(launch.message, "É necessária autenticação");
    assert_eq!(launch.identities, vec!["unix-user:alice".to_string(), "unix-user:bob".to_string()]);
    assert_eq!(launch.pipe_fd, 3);
    assert_eq!(launch.events_fd, 4);
}

#[test]
fn missing_flag_returns_usage_error() {
    // Flag faltante => Err com o uso (fail-secure, sem sucesso silencioso).
    let err = parse_polkit_args(&argv(&["--action-id", "org.x"])).unwrap_err();
    assert_eq!(err, POLKIT_USAGE);
}

#[test]
fn missing_value_and_bad_fd_return_usage_error() {
    let err = parse_polkit_args(&argv(&["--action-id"])).unwrap_err();
    assert_eq!(err, POLKIT_USAGE);
    let err = parse_polkit_args(&argv(&[
        "--action-id",
        "a",
        "--message",
        "m",
        "--identities",
        "u",
        "--pipe",
        "-1",
        "--events",
        "abc",
    ]))
    .unwrap_err();
    assert_eq!(err, POLKIT_USAGE);
}

#[test]
fn maps_wire_events() {
    assert_eq!(
        gui_event_from_json(r#"{"prompt":"Senha:","echo":false}"#),
        Some(GuiEvent::Prompt {
            prompt: "Senha:".to_string(),
            echo: false
        })
    );
    assert_eq!(
        gui_event_from_json(r#"{"prompt":"Password:","echo":true}"#),
        Some(GuiEvent::Prompt {
            prompt: "Password:".to_string(),
            echo: true
        })
    );
    assert_eq!(
        gui_event_from_json(r#"{"error":"Falha de autenticação"}"#),
        Some(GuiEvent::Error("Falha de autenticação".to_string()))
    );
    assert_eq!(
        gui_event_from_json(r#"{"info":"Tente novamente"}"#),
        Some(GuiEvent::Info("Tente novamente".to_string()))
    );
    assert_eq!(
        gui_event_from_json(r#"{"completed":true}"#),
        Some(GuiEvent::Completed(true))
    );
    assert_eq!(
        gui_event_from_json(r#"{"completed":false}"#),
        Some(GuiEvent::Completed(false))
    );
}

#[test]
fn ignores_malformed_lines_without_completing() {
    // Linha malformada => None, ignorada — e NUNCA um `Completed`.
    for line in [
        "",
        "not json",
        r#"{"completed":tru"#,
        r#"{"unknown":1}"#,
        r#"{"prompt":123}"#,
        r#"{"completed":"yes"}"#,
        "[1,2]",
        "null",
    ] {
        assert_eq!(gui_event_from_json(line), None, "{line:?}");
        assert!(
            !matches!(gui_event_from_json(line), Some(GuiEvent::Completed(_))),
            "{line:?}"
        );
    }
}

#[test]
fn submit_cancel_framing_is_byte_exact() {
    assert_eq!(submit_line("s3cr3t").as_deref(), Some("{\"password\":\"s3cr3t\"}\n"));
    assert_eq!(cancel_line(), "{\"cancel\":true}\n");
    // Senhas com escapes JSON sobrevivem ao round-trip, sempre com `\n` final.
    let tricky = "a\"b\\c";
    let line = submit_line(tricky).expect("sem quebra de linha");
    assert!(line.ends_with('\n'));
    let value: serde_json::Value = serde_json::from_str(line.trim_end()).unwrap();
    assert_eq!(value["password"], tricky);
    // Invariante PAM: senha com `\n` nunca é enviada.
    assert_eq!(submit_line("a\nb"), None);
}

#[test]
fn html_invokes_match_registered_commands() {
    // O `invoke_handler` registra `polkit_state|submit|cancel`; o HTML deve
    // invocar exatamente esses nomes (+ ouvir `polkit-event`).
    const HTML: &str = include_str!("../../src/routes/system/polkit.html");
    for name in ["polkit_state", "polkit_submit", "polkit_cancel", "polkit-event"] {
        assert!(HTML.contains(name), "{name}");
    }
    // HIG: autofocus + Enter confirma / Esc cancela.
    assert!(HTML.contains("autofocus"), "autofocus");
    assert!(HTML.contains("'Enter'"), "Enter");
    assert!(HTML.contains("Escape"), "Esc");
    // Display sem prefixo `unix-user:` (senha vale p/ a primeira identity).
    assert!(HTML.contains("unix-user:"), "display");
    // Tokens dark/light (paridade `portal.html`).
    assert!(HTML.contains("[data-theme=\"light\"]"), "light");
    assert!(HTML.contains("--window-bg"), "tokens");
    // Toggle de visibilidade: olho SVG dentro do input (sem texto —
    // `renderEcho` nunca usa `textContent` no botão, que destruiria o SVG).
    assert!(HTML.contains("eye-open") && HTML.contains("eye-off"), "eye svgs");
    assert!(HTML.contains("aria-label=\"Mostrar senha\""), "eye label");
    assert!(!HTML.contains("toggleEl.textContent"), "no textContent on toggle");
}
