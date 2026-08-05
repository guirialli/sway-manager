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
