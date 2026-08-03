# Spec 06: Atualização do Instalador Raiz e Remoção Completa do Python 🧹🦀

## 1. Objetivos da Etapa
1. Atualizar o script de instalação da raiz do repositório (`/home/guilherme/projetos/sway-config/install.sh`).
2. Remover por completo todo o código legado em Python, ambientes virtuais (`venv/`), pastas de testes e artefatos de compilação Nuitka.
3. Garantir que a instalação do ecossistema Sway dependa exclusivamente do compilador Rust (`cargo`).

---

## 2. Modificação do Instalador Raiz (`/home/guilherme/projetos/sway-config/install.sh`)

### Alterações na Função `install_sway_manager()`:
- **Remoção de Venv/Pip**: Eliminar a criação de ambiente virtual Python (`python3 -m venv sway-manager-venv`) e instalação de pacotes via `pip install PySide6 ...`.
- **Compilação Rust Nativa**:
  ```bash
  install_sway_manager() {
      print_msg "Compilando e instalando SwayManager nativo em Rust..."
      cd "$SWAY_MANAGER_DIR"
      
      if ! command -v cargo &> /dev/null; then
          print_error "Cargo/Rust não encontrado! Instale com: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
          exit 1
      fi

      cargo build --release --workspace

      INSTALL_BIN_DIR="$HOME/.config/sway/bin"
      mkdir -p "$INSTALL_BIN_DIR"

      cp target/release/sway-manager-cli "$INSTALL_BIN_DIR/SwayManager"
      cp target/release/sway-manager-daemon "$INSTALL_BIN_DIR/sway-manager-daemon"
      chmod +x "$INSTALL_BIN_DIR/SwayManager" "$INSTALL_BIN_DIR/sway-manager-daemon"

      print_success "SwayManager em Rust instalado com sucesso em $INSTALL_BIN_DIR!"
  }
  ```

---

## 3. Limpeza Completa do Código Legado em Python

Após a conclusão e validação da implementação em Rust, os seguintes diretórios e arquivos Python serão **permanentemente removidos**:

| Diretório / Arquivo | Ação |
|---|---|
| `sway-manager/src/` (Python) | Removido (substituído por `sway-manager/crates/`) |
| `sway-manager/tests/` (Python) | Removido (substituído por `cargo test`) |
| `sway-manager/venv/` | Removido |
| `sway-manager/out/` e `build/` (Nuitka) | Removidos |
| `sway-manager/requirements.txt` | Removido |
| `sway-manager/install.sh` (Script antigo) | Atualizado para compilação Cargo |
| `/home/guilherme/projetos/sway-config/src/sway/bin/sway-manager-src/` | Removido |

---

## 4. Atualização das Configurações do Sway (`30-autostart`)
Garantir que a inicialização no Sway execute o binário Rust em background:
```swayconfig
exec $HOME/.config/sway/bin/sway-manager-daemon &
```
