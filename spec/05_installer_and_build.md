# Spec 05: Script de Instalação e Configuração do Cargo Build (`install.sh`) 🛠️

## 1. Otimizações de Compilação no `Cargo.toml`

Para garantir o menor tamanho de binário possível e máxima performance de execução:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

---

## 2. Script de Instalação (`install.sh`)

O script de instalação raiz do projeto será atualizado para compilar a versão Rust:

```bash
#!/usr/bin/env bash
set -e

echo "📦 Compilando SwayManager em Rust (Modo Release)..."
cargo build --release --workspace

INSTALL_DIR="$HOME/.config/sway/bin"
mkdir -p "$INSTALL_DIR"

cp target/release/sway-manager-cli "$INSTALL_DIR/SwayManager"
cp target/release/sway-manager-daemon "$INSTALL_DIR/sway-manager-daemon"

echo "✅ Compilação e instalação concluídas em $INSTALL_DIR/SwayManager!"
```

---

## 3. Integração com Autostart do Sway
O arquivo `~/.config/sway/config.d/30-autostart` inicializa o daemon em background:
```swayconfig
exec $HOME/.config/sway/bin/sway-manager-daemon &
```
