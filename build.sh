#!/bin/bash

# Exit on error
set -e

DAEMON_APP_NAME="SwayManager"
GUI_APP_NAME="SwayManagerGUI"
GUI_CRATE_DIR="gui/src-tauri"
OUTPUT_DIR="out"

echo "--- 🚀 Iniciando processo de Build (Daemon Rust + GUI Tauri) ---"

# Verificar se o cargo está disponível (build do daemon e da GUI)
if ! command -v cargo &> /dev/null; then
    echo "❌ Erro: cargo não encontrado. Instale o Rust com: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Verificar dev-libs do Tauri/WebKitGTK
if ! pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
    echo "❌ Erro: libwebkit2gtk-4.1-dev não encontrada."
    echo "💡 Instale com: sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev"
    exit 1
fi

echo "🦀 Executando suíte de testes do daemon (Rust)..."
(
    cd daemon
    cargo test
)

echo "🦀 Executando suíte de testes da GUI (Rust)..."
(
    cd "$GUI_CRATE_DIR"
    cargo test
)

echo "🧹 Limpando pastas de build anteriores..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo "⚙️ Compilando o daemon/CLI ($DAEMON_APP_NAME) com Cargo (release)..."
(
    cd daemon
    cargo build --release
)
cp "daemon/target/release/sway-manager-daemon" "$OUTPUT_DIR/$DAEMON_APP_NAME"

echo "⚙️ Compilando a GUI Tauri ($GUI_APP_NAME) com Cargo (release)..."
(
    cd "$GUI_CRATE_DIR"
    # O build-script do Tauri só observa tauri.conf.json: sem o touch,
    # HTML/CSS/JS alterados NÃO são re-embarcados no binário (bug silencioso).
    touch tauri.conf.json
    cargo build --release
)
cp "$GUI_CRATE_DIR/target/release/$GUI_APP_NAME" "$OUTPUT_DIR/$GUI_APP_NAME"

echo "📦 Agrupando binários em $OUTPUT_DIR/main.dist/..."
mkdir -p "$OUTPUT_DIR/main.dist"
cp "$OUTPUT_DIR/$DAEMON_APP_NAME" "$OUTPUT_DIR/main.dist/"
cp "$OUTPUT_DIR/$GUI_APP_NAME" "$OUTPUT_DIR/main.dist/"

# Criar link simbólico sway-manager e sway-manager-gui no out/main.dist
cd "$OUTPUT_DIR/main.dist"
ln -sf "$DAEMON_APP_NAME" sway-manager
ln -sf "$GUI_APP_NAME" sway-manager-gui
cd - > /dev/null

if [ -f "$OUTPUT_DIR/main.dist/$DAEMON_APP_NAME" ] && [ -f "$OUTPUT_DIR/main.dist/$GUI_APP_NAME" ]; then
    echo "--- ✅ Build concluído com sucesso! ---"
    echo "📍 Binários compilados disponíveis em:"
    echo "   - Daemon/CLI (Rust): ./$OUTPUT_DIR/main.dist/$DAEMON_APP_NAME"
    echo "   - GUI (Tauri):       ./$OUTPUT_DIR/main.dist/$GUI_APP_NAME"
else
    echo "❌ Erro durante a compilação."
    exit 1
fi
