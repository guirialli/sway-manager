#!/bin/bash

# Exit on error
set -e

VENV_PATH="venv"
MAIN_SOURCE="src/main.py"
GUI_SOURCE="src/gui_main.py"
MAIN_APP_NAME="SwayManager"
GUI_APP_NAME="SwayManagerGUI"
OUTPUT_DIR="out"

echo "--- 🚀 Iniciando processo de Build com Nuitka (SwayManager + SwayManagerGUI) ---"

# Verificar se o Python 3 está disponível
if ! command -v python3 &> /dev/null; then
    echo "❌ Erro: python3 não encontrado. Instale o Python com: sudo apt install python3 python3-venv"
    exit 1
fi

# Verificar se o patchelf está disponível (necessário para o Nuitka em modo standalone no Linux)
if ! command -v patchelf &> /dev/null; then
    echo "❌ Erro: 'patchelf' não encontrado. O Nuitka requer 'patchelf' para compilar em modo standalone no Linux."
    echo "💡 Instale com: sudo apt install patchelf  (ou sudo dnf install patchelf / nix-shell)"
    exit 1
fi

# Criar venv se não existir
if [ ! -d "$VENV_PATH" ]; then
    echo "📦 Criando ambiente virtual em $VENV_PATH..."
    python3 -m venv "$VENV_PATH"
fi

echo "🔄 Ativando ambiente virtual..."
source "$VENV_PATH/bin/activate"

echo "⬆️ Atualizando pip e instalando dependências do projeto..."
pip install --upgrade pip setuptools wheel
if [ -f "requirements.txt" ]; then
    pip install -r requirements.txt
fi

if ! command -v nuitka &> /dev/null; then
    echo "📦 Nuitka não encontrado no ambiente virtual. Instalando..."
    pip install nuitka
fi

echo "🧪 Executando suíte de testes automatizados..."
PYTHONPATH=src python3 -m unittest discover -s tests -v

echo "🧹 Limpando pastas de build anteriores..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

NUITKA_COMMON_ARGS=(
    "--standalone"
    "--enable-plugin=pyside6"
    "--assume-yes-for-downloads"
    "--warn-implicit-exceptions"
)

echo "⚙️ Compilando o executável principal ($MAIN_APP_NAME) com Nuitka..."
python3 -m nuitka "${NUITKA_COMMON_ARGS[@]}" "--output-dir=$OUTPUT_DIR" "--output-filename=$MAIN_APP_NAME" "$MAIN_SOURCE"

echo "⚙️ Compilando o executável GUI ($GUI_APP_NAME) com Nuitka..."
python3 -m nuitka "${NUITKA_COMMON_ARGS[@]}" "--output-dir=$OUTPUT_DIR" "--output-filename=$GUI_APP_NAME" "$GUI_SOURCE"

echo "📦 Agrupando binários em $OUTPUT_DIR/main.dist/..."
if [ -d "$OUTPUT_DIR/gui_main.dist" ]; then
    cp -rf "$OUTPUT_DIR/gui_main.dist/"* "$OUTPUT_DIR/main.dist/"
    rm -rf "$OUTPUT_DIR/gui_main.dist"
fi

# Criar link simbólico sway-manager e sway-manager-gui no out/main.dist
cd "$OUTPUT_DIR/main.dist"
ln -sf "$MAIN_APP_NAME" sway-manager
ln -sf "$GUI_APP_NAME" sway-manager-gui
cd - > /dev/null

if [ -f "$OUTPUT_DIR/main.dist/$MAIN_APP_NAME" ] && [ -f "$OUTPUT_DIR/main.dist/$GUI_APP_NAME" ]; then
    echo "--- ✅ Build concluído com sucesso! ---"
    echo "📍 Binários compilados disponíveis em:"
    echo "   - Main/Daemon: ./$OUTPUT_DIR/main.dist/$MAIN_APP_NAME"
    echo "   - GUI:         ./$OUTPUT_DIR/main.dist/$GUI_APP_NAME"
else
    echo "❌ Erro durante a compilação com Nuitka."
    exit 1
fi

deactivate
