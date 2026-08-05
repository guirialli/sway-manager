#!/usr/bin/env bash
set -e

echo "📦 Compilando SwayManager em Rust (Modo Release)..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Erro: Cargo/Rust não encontrado! Instale com: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

cargo build --release --workspace

INSTALL_DIR="$HOME/.config/sway/bin"
mkdir -p "$INSTALL_DIR"

cp -f target/release/SwayManager "$INSTALL_DIR/SwayManager" 2>/dev/null || cp -f target/release/sway-manager-cli "$INSTALL_DIR/SwayManager"
cp -f target/release/sway-manager-daemon "$INSTALL_DIR/sway-manager-daemon"
chmod +x "$INSTALL_DIR/SwayManager" "$INSTALL_DIR/sway-manager-daemon"

echo "✅ Compilação e instalação concluídas em $INSTALL_DIR/SwayManager!"

INSTALL_UDEV=true
if [[ "$1" == "--no-udev" || "$1" == "--skip-udev" ]]; then
    INSTALL_UDEV=false
fi

if [ "$INSTALL_UDEV" = true ]; then
    echo "Instalando regras udev para conservação de bateria (requer sudo)..."
    if [ -f ./udev/99-battery-conservation.rules ]; then
        sudo cp ./udev/99-battery-conservation.rules /etc/udev/rules.d/
        sudo udevadm control --reload-rules
        sudo udevadm trigger
        echo "Regras udev instaladas com sucesso!"
    fi
fi
