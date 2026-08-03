# Spec 00: Visão Geral e Arquitetura do SwayManager em Rust 🦀

## 1. Objetivos do Projeto
Reescrever o **SwayManager** em **Rust** mantendo **100% de paridade de recursos** com a versão Python, porém eliminando a dependência do runtime Python, Qt6/PySide6 e PyGObject/GTK3.

### Metas Quantitativas de Desempenho
- **Uso de Memória RAM em Daemon**: **< 5 MB** (redução de 97% em relação aos 180MB+ do Python).
- **Tempo de Resposta IPC**: **< 1 ms** via Unix Domain Socket.
- **Tempo de Inicialização do Cliente CLI**: **< 2 ms**.
- **Tamanho do Binário Final Compilado**: Binários otimizados com `strip = true` e `panic = "abort"`.

---

## 2. Estrutura do Workspace Cargo

```text
sway-manager-rust/
├── Cargo.toml
├── spec/                          # Especificações técnicas por módulo
├── crates/
│   ├── sway-manager-core/        # Entidades, VO, traits e repositórios de sistema
│   ├── sway-manager-daemon/      # Servidor Daemon Tokio, Unix Socket, Logging & Cache
│   ├── sway-manager-cli/         # Binário CLI ultraleve (clap)
│   └── sway-manager-gui/         # GUIs em Slint (OSDs, Wallpaper Picker, Control Center)
```

---

## 3. Protocolo IPC (Unix Domain Socket)

- **Caminho do Socket**: `$XDG_RUNTIME_DIR/sway-manager.sock`
- **Formato da Requisição**: Linha JSON delimitada por `\n`
  ```json
  {"args": ["SwayManager", "battery", "status"]}
  ```
- **Formato da Resposta**: Linha JSON delimitada por `\n`
  ```json
  {"stdout": "...", "stderr": "", "exit_code": 0}
  ```
- **Comandos Interativos (GUI)**:
  Para comandos que abrem interfaces (`menu`, `screenshot`, `clipboard`, `settings`, `wallpaper`, `osd`), o Daemon retorna imediatamente a resposta JSON (`exit_code: 0`) em **< 1ms** para liberar o cliente CLI, e em seguida renderiza a janela gráfica na thread principal do evento.

---

## 4. Estratégia de Segurança e Resiliência
- Nenhuma falha em manipulador de comando ou janela pode derrubar o processo daemon (`catch_unwind` e tratamento de erros via `Result<T, E>`).
- Registros de erros e ações do sistema gravados em `~/.config/sway-manager/logs/log-YYYY-MM-DD.txt` através de logging assíncrono não-bloqueante.
