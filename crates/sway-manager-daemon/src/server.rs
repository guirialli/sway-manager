use crate::cache::ApplicationCache;
use crate::dispatcher::{CommandDispatcher, IpcRequest, IpcResponse};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"));
    runtime_dir.join("sway-manager.sock")
}

pub struct SingleInstanceGuard;

impl SingleInstanceGuard {
    pub async fn check_or_acquire() -> Result<(), String> {
        let socket_path = get_socket_path();
        if socket_path.exists() {
            if UnixStream::connect(&socket_path).await.is_ok() {
                return Err("Outra instância do SwayManager Daemon já está em execução.".to_string());
            } else {
                info!("Socket residual encontrado em {:?}. Removendo...", socket_path);
                let _ = tokio::fs::remove_file(&socket_path).await;
            }
        }
        Ok(())
    }
}

pub async fn run_server() -> Result<(), String> {
    SingleInstanceGuard::check_or_acquire().await?;

    let socket_path = get_socket_path();
    let listener = UnixListener::bind(&socket_path)
        .map_err(|e| format!("Não foi possível registrar o socket em {:?}: {}", socket_path, e))?;

    info!("SwayManager Daemon ouvindo em {:?}", socket_path);

    let cache = ApplicationCache::new();
    let dispatcher = std::sync::Arc::new(CommandDispatcher::new(cache));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let disp = dispatcher.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, &disp).await {
                        error!("Erro ao processar conexão IPC: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Erro no accept do socket: {}", e);
            }
        }
    }
}

async fn handle_connection(stream: UnixStream, dispatcher: &CommandDispatcher) -> Result<(), String> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    if buf_reader.read_line(&mut line).await.map_err(|e| e.to_string())? > 0 {
        let response = match serde_json::from_str::<IpcRequest>(&line) {
            Ok(req) => dispatcher.dispatch(req),
            Err(e) => IpcResponse {
                stdout: String::new(),
                stderr: format!("Formato de requisição JSON inválido: {}", e),
                exit_code: 1,
            },
        };

        let json_resp = serde_json::to_string(&response).map_err(|e| e.to_string())? + "\n";
        writer.write_all(json_resp.as_bytes()).await.map_err(|e| e.to_string())?;
        writer.flush().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}
