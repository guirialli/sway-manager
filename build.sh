#!/bin/bash

# Exit on error
set -e

VENV_PATH="venv"
SOURCE_FILE="src/main.py"
APP_NAME="sway-manager"
OUTPUT_DIR="out"

echo "--- 🚀 Iniciando processo de Build com Nuitka (Debian/Ubuntu/Linux) ---"

# Verificar se o Python 3 está disponível
if ! command -v python3 &> /dev/null; then
    echo "❌ Erro: python3 não encontrado. Instale o Python com: sudo apt install python3 python3-venv"
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

# Argumentos do Nuitka para Debian / Linux padrão
NUITKA_ARGS=(
    "--standalone"
    "--output-dir=$OUTPUT_DIR"
    "--output-filename=$APP_NAME"
    "--enable-plugin=pyside6"
    "--assume-yes-for-downloads"
    "--warn-implicit-exceptions"
)

echo "⚙️ Traduzindo Python para C++ e compilando com Nuitka (pode levar alguns minutos)..."
python3 -m nuitka "${NUITKA_ARGS[@]}" "$SOURCE_FILE"

if [ $? -eq 0 ]; then
    echo "--- ✅ Build concluído com sucesso! ---"
    echo "📍 O seu executável compilado está em: ./$OUTPUT_DIR/main.dist/$APP_NAME"
else
    echo "❌ Erro durante a compilação com Nuitka."
    exit 1
fi

deactivate
