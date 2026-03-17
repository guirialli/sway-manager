#!/bin/bash
set -e

echo "Compilando o projeto..."
./build.sh


echo "Instalando o sway-manager"
rm -rf ~/.config/sway/bin/sway-manager
rm ~/.config/sway/bin/SwayManager
mkdir -p ~/.config/sway/bin/sway-manager
cp -rf ./out/main.dist/* ~/.config/sway/bin/sway-manager/
ln -s $HOME/.config/sway/bin/sway-manager/sway-manager $HOME/.config/sway/bin/SwayManager 
