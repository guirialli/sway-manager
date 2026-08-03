# Spec 02: Servidor Daemon e Servidor IPC Socket (`sway-manager-daemon`) 🚀

## 1. Responsabilidade da Crate
Servidor em segundo plano construído com `tokio` que roda continuamente com **< 5 MB de RAM**.

---

## 2. Componentes Principais

```text
crates/sway-manager-daemon/src/
├── main.rs
├── server.rs            # Listener Unix Domain Socket tokio::net::UnixListener
├── cache.rs             # Cache em memória RAM de aplicativos e ícones (Arc<RwLock<...>>)
├── logger.rs            # Logger assíncrono com rotação diária de arquivos
└── dispatcher.rs        # Roteamento e despacho de comandos recebidos no socket
```

---

## 3. Especificações Técnicas

### A. Proteção de Instância Única (Single Instance Guard)
- Antes de iniciar o servidor socket em `$XDG_RUNTIME_DIR/sway-manager.sock`, o daemon tenta conectar no socket existente.
- Se houver resposta de um daemon ativo, o processo encerra imediatamente com mensagem explicativa.
- Se o socket for um arquivo residual inativo, remove o arquivo e assume a escuta.

### B. Cache de Aplicativos Wofi em RAM (`cache.rs`)
- Carrega e analisa todos os arquivos `.desktop` de `/usr/share/applications/` e `~/.local/share/applications/` utilizando `freedesktop-desktop-entry`.
- Armazena as entradas formatadas e caminhos de ícones em uma estrutura `Arc<RwLock<Vec<MenuItem>>>`.
- Permite o carregamento instantâneo do menu Wofi sem ler o disco a cada acionamento.

### C. Logging Assíncrono Não-Bloqueante (`logger.rs`)
- Gravação assíncrona usando `tracing` e `tracing-appender`.
- Arquivos salvos em `~/.config/sway-manager/logs/log-YYYY-MM-DD.txt`.
- O log pode ser monitorado pelo comando `sway-manager daemon log -f`.

### D. Tratamento Global de Panics e Resiliência
- Define `std::panic::set_hook` para capturar qualquer exceção ou panic em threads secundárias, registrando o erro no log **sem derrubar a execução do daemon principal**.
