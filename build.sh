#!/bin/bash

VENV_PATH="venv"
SOURCE_FILE="src/main.py"
APP_NAME="sway-manager"
OUTPUT_DIR="out"

echo "--- Iniciando processo de Build com NUITKA (NixOS Blindado) ---"

# Trava de segurança para garantir o nix-shell
if [ -z "$IN_NIX_SHELL" ]; then
    echo "❌ ERRO: Você precisa rodar este script DENTRO do nix-shell!"
    exit 1
fi

if [ -d "$VENV_PATH" ]; then
    echo "Ativando ambiente virtual..."
    source "$VENV_PATH/bin/activate"
else
    echo "Erro: Ambiente virtual não encontrado em $VENV_PATH"
    exit 1
fi

if ! command -v nuitka &> /dev/null; then
    echo "Nuitka não encontrado. Instalando..."
    pip install nuitka
fi

echo "Limpando pastas de build antigas..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Lista consolidada de TUDO que o PySide6/Wayland/X11 precisa no NixOS
LIBS_TO_COPY=(
    "libxcb-cursor.so.0"
    "libwayland-client.so.0"
    "libwayland-egl.so.1"
    "libwayland-cursor.so.0"
    "libxkbcommon.so.0"
    "libEGL.so.1"
    "libGL.so.1"
    "libGLX.so.0" 
    "libGLdispatch.so.0" 
    "libfontconfig.so.1"
    "libfreetype.so.6"
    "libgssapi_krb5.so.2"
    "libbrotlidec.so.1"
    "libzstd.so.1"
    "libz.so.1"    
    "libstdc++.so.6"
    "libgcc_s.so.1"
    "libX11.so.6"
    "libglib-2.0.so.0"
    "libgthread-2.0.so.0"
    "libdbus-1.so.3"
)

# Argumentos base do Nuitka (Limpos, sem incluir arquivos do SO aqui)
NUITKA_ARGS=(
    "--standalone"
    "--output-dir=$OUTPUT_DIR"
    "--output-filename=$APP_NAME"
    "--enable-plugin=pyside6"
    "--assume-yes-for-downloads"
    "--warn-implicit-exceptions"
)

echo "⚙️ Traduzindo para C++ e compilando (Isso pode demorar alguns minutos)..."

# Roda o módulo do Nuitka pelo Python
python -m nuitka "${NUITKA_ARGS[@]}" "$SOURCE_FILE"

if [ $? -eq 0 ]; then
    echo "--- Build base concluído! ---"
    
    echo "🔧 Injetando TODAS as bibliotecas do NixOS fisicamente na pasta .dist..."
    
    # Transforma o LD_LIBRARY_PATH em um array de pastas
    IFS=':' read -r -a LIB_DIRS <<< "$LD_LIBRARY_PATH"
    
    # Fazemos a cópia na força bruta, resolvendo symlinks com 'cp -L'
    for lib in "${LIBS_TO_COPY[@]}"; do
        FOUND=false
        for dir in "${LIB_DIRS[@]}"; do
            if [ -e "$dir/$lib" ]; then
                # cp -L  copia o arquivo real e ignora que é um atalho do Nix
                cp -L "$dir/$lib" "./$OUTPUT_DIR/main.dist/"
                echo "  ✅ Injetado fisicamente: $lib"
                FOUND=true
                break
            fi
        done
        if [ "$FOUND" = false ]; then
            echo "  ⚠️ Aviso Crítico: Não encontrei $lib no LD_LIBRARY_PATH do shell!"
        fi
    done

    echo "--- Build 100% concluído! ---"
    echo "📍 O seu executável super rápido está em: ./$OUTPUT_DIR/main.dist/$APP_NAME"
else
    echo "❌ Erro durante a compilação com Nuitka."
    exit 1
fi

deactivate
