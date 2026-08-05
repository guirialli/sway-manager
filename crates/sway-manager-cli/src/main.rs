use serde::Deserialize;
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::exit;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Deserialize)]
struct IpcResponse {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"));
    runtime_dir.join("sway-manager.sock")
}

fn show_help() {
    let help_text = r#"
SwayManager - Suite de Gerenciamento para Sway e SwayFX em Rust 🦀

Uso:
  SwayManager <comando> [opções]

Comandos Disponíveis:
  daemon                   Inicia o servidor daemon persistente em background.
  daemon log [-f|--follow] Exibe ou acompanha em tempo real os logs do dia (~/.config/sway-manager/logs/).
  settings, config         Abre o painel gráfico do Control Center (Configurações).
  monitor                  Abre a janela gráfica para alternar layouts de monitores.
  wallpaper [picker|set <caminho>] Seleciona ou define papel de parede.
  osd brilho [up|down]     Ajusta o brilho e exibe o OSD gráfico.
  osd volume [up|down|mute] Ajusta o volume e exibe o OSD gráfico.
  battery [toggle|status]  Alterna a conservação de bateria (~80% vs 100%) ou retorna status JSON.
  idle [toggle|status]     Alterna o inibidor de suspensão swayidle ou retorna status JSON.
  theme [toggle|status]    Alterna o tema (Dark/Light) do GTK/Qt ou retorna status.
  power [toggle|status|profile <nome>] Alterna ou define perfil de energia.
  screenshot [full|area|window] Tira captura de tela e copia para clipboard.
  menu [filtro]            Dispara o menu Wofi pré-carregado em RAM.
  clipboard [clear|menu]   Gerenciador de área de transferência.
  lock                     Bloqueia a tela usando swaylock.
  -h, --help               Exibe esta mensagem de ajuda.
"#;
    println!("{}", help_text.trim());
}

fn handle_daemon_log(args: &[String]) {
    let mut log_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    log_dir.push("sway-manager");
    log_dir.push("logs");

    if !log_dir.exists() {
        println!("Nenhum log encontrado em {:?}", log_dir);
        return;
    }

    let follow = args.iter().any(|arg| arg == "-f" || arg == "--follow");

    // Read entries in log dir and pick newest
    if let Ok(entries) = std::fs::read_dir(&log_dir) {
        let mut log_files: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        log_files.sort();

        if let Some(latest) = log_files.last() {
            if let Ok(file) = File::open(latest) {
                let mut reader = BufReader::new(file);
                let mut lines = Vec::new();
                let mut line = String::new();

                while reader.read_line(&mut line).unwrap_or(0) > 0 {
                    lines.push(line.clone());
                    line.clear();
                }

                if !follow {
                    let tail = if lines.len() > 50 { &lines[lines.len() - 50..] } else { &lines[..] };
                    for l in tail {
                        print!("{}", l);
                    }
                    return;
                }

                for l in &lines {
                    print!("{}", l);
                }
                println!("--- Acompanhando logs em tempo real ({:?}) [Ctrl+C para sair] ---", latest);

                loop {
                    line.clear();
                    if reader.read_line(&mut line).unwrap_or(0) > 0 {
                        print!("{}", line);
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
            }
        }
    }
    println!("Nenhum log gravado para hoje.");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        show_help();
        return;
    }

    let cmd = args[1].to_lowercase();
    if cmd == "-h" || cmd == "--help" || cmd == "help" {
        show_help();
        return;
    }

    if cmd == "daemon" || cmd == "--daemon" || cmd == "-d" {
        if args.len() > 2 && (args[2] == "log" || args[2] == "logs" || args[2] == "-l") {
            handle_daemon_log(&args[2..]);
            return;
        }

        // Spawn daemon binary
        let status = std::process::Command::new("sway-manager-daemon")
            .status();

        match status {
            Ok(s) => exit(s.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("❌ Erro ao iniciar o daemon: {}", e);
                exit(1);
            }
        }
    }

    let socket_path = get_socket_path();
    let stream = match tokio::net::UnixStream::connect(&socket_path).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "❌ Erro: O SwayManager Daemon não está em execução em {:?}.\nInicie o serviço com 'SwayManager daemon' ou verifique a inicialização do Sway.",
                socket_path
            );
            exit(1);
        }
    };

    let (reader, mut writer) = stream.into_split();
    let payload = json!({ "args": args });
    let mut json_str = payload.to_string();
    json_str.push('\n');

    if let Err(e) = writer.write_all(json_str.as_bytes()).await {
        eprintln!("Erro ao enviar solicitação IPC: {}", e);
        exit(1);
    }
    let _ = writer.flush().await;

    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut resp_line = String::new();
    if tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut resp_line).await.is_ok() {
        if let Ok(resp) = serde_json::from_str::<IpcResponse>(&resp_line) {
            if !resp.stdout.is_empty() {
                println!("{}", resp.stdout);
            }
            if !resp.stderr.is_empty() {
                eprintln!("{}", resp.stderr);
            }
            exit(resp.exit_code);
        }
    }

    eprintln!("Erro na comunicação IPC com o daemon");
    exit(1);
}
