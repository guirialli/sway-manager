mod cache;
mod dispatcher;
mod logger;
mod server;

use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Setup panic hook for resilience
    std::panic::set_hook(Box::new(|info| {
        error!("SwayManager Daemon panic capturado: {}", info);
    }));

    if let Err(e) = logger::init_logger() {
        eprintln!("Aviso: Não foi possível inicializar o logger assíncrono: {}", e);
    }

    info!("Iniciando SwayManager Daemon em Rust...");

    if let Err(e) = server::run_server().await {
        eprintln!("❌ Erro fatal no daemon: {}", e);
        error!("Erro fatal no daemon: {}", e);
        std::process::exit(1);
    }
}
