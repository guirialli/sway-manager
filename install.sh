#!/bin/bash
set -e

echo "Compilando o projeto..."
./build.sh

echo "Instalando o SwayManager e SwayManagerGUI..."
rm -rf ~/.config/sway/bin/sway-manager
rm -f ~/.config/sway/bin/SwayManager
rm -f ~/.config/sway/bin/sway-manager
rm -f ~/.config/sway/bin/SwayManagerGUI
rm -f ~/.config/sway/bin/sway-manager-gui
rm -f ~/.config/sway/bin/SwayManagerDaemon

mkdir -p ~/.config/sway/bin/sway-manager
cp -rf ./out/main.dist/* ~/.config/sway/bin/sway-manager/

ln -sf $HOME/.config/sway/bin/sway-manager/SwayManager $HOME/.config/sway/bin/SwayManager
ln -sf $HOME/.config/sway/bin/sway-manager/sway-manager $HOME/.config/sway/bin/sway-manager
ln -sf $HOME/.config/sway/bin/sway-manager/SwayManagerGUI $HOME/.config/sway/bin/SwayManagerGUI
ln -sf $HOME/.config/sway/bin/sway-manager/sway-manager-gui $HOME/.config/sway/bin/sway-manager-gui

INSTALL_UDEV=true
INSTALL_LSWT=false

for arg in "$@"; do
    case "$arg" in
        --no-udev|--skip-udev)
            INSTALL_UDEV=false
            ;;
        --with-lswt|--install-lswt)
            INSTALL_LSWT=true
            ;;
    esac
done

PYTHON_BIN="python3"
if [ -x "venv/bin/python3" ]; then
    PYTHON_BIN="venv/bin/python3"
fi

echo "Configurando XDG Desktop Portal..."
PYTHONPATH=src $PYTHON_BIN - <<'PY'
from portal.config_installer import PortalConfigInstaller
import sys

try:
    installer = PortalConfigInstaller()
    log = installer.install()
    for line in log:
        print(f"  {line}")
except Exception as exc:
    print(f"Aviso: falha ao configurar portal: {exc}", file=sys.stderr)
PY

if [ "$INSTALL_UDEV" = true ]; then
    echo "Instalando regras udev para conservação de bateria (requer sudo)..."
    if [ -f ./udev/99-battery-conservation.rules ]; then
        sudo cp ./udev/99-battery-conservation.rules /etc/udev/rules.d/
        sudo udevadm control --reload-rules
        sudo udevadm trigger
        echo "Regras udev instaladas com sucesso!"
    fi
fi

echo "Verificando utilitário lswt (para compartilhamento de janelas)..."
if command -v lswt &>/dev/null; then
    echo "  lswt já está instalado: $(command -v lswt)"
elif [ "$INSTALL_LSWT" = true ]; then
    echo "Instalando lswt..."
    TMP_LSWT=$(mktemp -d)
    echo "Clonando e compilando lswt (v2.0.0) em $TMP_LSWT..."
    git clone https://git.sr.ht/~leon_plickat/lswt "$TMP_LSWT"
    (
        cd "$TMP_LSWT"
        git checkout v2.0.0
        make
        sudo make install
    )
    rm -rf "$TMP_LSWT"
    echo "lswt instalado com sucesso!"
else
    echo "  Aviso: lswt não encontrado. O compartilhamento de tela funcionará para monitores."
    echo "  Para instalar o lswt automaticamente, execute: ./install.sh --install-lswt"
fi

XDPW_BIN=/usr/libexec/xdg-desktop-portal-wlr
if [ -x "$XDPW_BIN" ] && strings "$XDPW_BIN" 2>/dev/null | grep -q "^Window: %s"; then
    echo "  xdg-desktop-portal-wlr já suporta captura de janela (>= 0.8.0)"
else
    echo "  Versão atual não suporta captura de janela; construindo v0.8.4 a partir do código-fonte..."
    TMP_XDPW=$(mktemp -d)
    git clone --branch v0.8.4 --depth 1 https://github.com/emersion/xdg-desktop-portal-wlr.git "$TMP_XDPW" 2>&1 | tail -1
    meson setup "$TMP_XDPW-build" "$TMP_XDPW" --prefix=/usr -Dsd-bus-provider=libsystemd 2>&1 | tail -3
    ninja -C "$TMP_XDPW-build" 2>&1 | tail -3
    sudo ninja -C "$TMP_XDPW-build" install 2>&1 | tail -5
    rm -rf "$TMP_XDPW" "$TMP_XDPW-build"
    echo "  xdg-desktop-portal-wlr v0.8.4 instalado com sucesso!"
fi

echo "Instalação concluída."
