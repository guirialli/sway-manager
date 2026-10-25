#!/bin/bash
# SwayManager installer: build, install binaries/symlinks, portal, udev,
# lswt, xdg-desktop-portal-wlr (patched) and the native polkit agent.
#
# Usage: ./install.sh [--no-udev] [--install-lswt] [--install-polkit]
#   --no-udev|--skip-udev        skip battery udev rules (no sudo needed)
#   --with-lswt|--install-lswt   build + install lswt (window sharing)
#   --install-polkit|--with-polkit  install native polkit agent (replaces rivals)
#   --no-polkit|--skip-polkit    never install the polkit agent (no prompt)
# Without polkit flags and on a TTY, the script asks (default No).
set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_UDEV=true
INSTALL_LSWT=false
INSTALL_POLKIT=false
POLKIT_ANSWERED=false

parse_args() {
    for arg in "$@"; do
        case "$arg" in
            --no-udev|--skip-udev)
                INSTALL_UDEV=false
                ;;
            --with-lswt|--install-lswt)
                INSTALL_LSWT=true
                ;;
            --install-polkit|--with-polkit)
                INSTALL_POLKIT=true
                ;;
            --no-polkit|--skip-polkit)
                INSTALL_POLKIT=false
                POLKIT_ANSWERED=true
                ;;
        esac
    done

    # No explicit polkit flag + interactive terminal: ask (default No —
    # swapping the polkit agent touches system authentication).
    if [ "$INSTALL_POLKIT" != true ] && [ "$POLKIT_ANSWERED" != true ] && [ -t 0 ]; then
        printf 'Deseja instalar o agente Polkit nativo do SwayManager (substitui lxpolkit/lxqt/mate/gnome)? [s/N] '
        read -r POLKIT_REPLY || true
        case "$POLKIT_REPLY" in
            [sSyY]*) INSTALL_POLKIT=true ;;
        esac
    fi
}

build_project() {
    echo "Compilando o projeto..."
    ./build.sh
}

install_binaries() {
    echo "Instalando o SwayManager (daemon Rust) e SwayManagerGUI (Tauri)..."
    rm -rf ~/.config/sway/bin/sway-manager
    rm -f ~/.config/sway/bin/SwayManager
    rm -f ~/.config/sway/bin/SwayManagerGUI
    rm -f ~/.config/sway/bin/sway-manager-gui
    rm -f ~/.config/sway/bin/SwayManagerDaemon

    mkdir -p ~/.config/sway/bin/sway-manager
    cp -rf ./out/main.dist/* ~/.config/sway/bin/sway-manager/

    # SwayManager = daemon Rust; SwayManagerGUI = GUI Tauri (sway-manager-gui = alias).
    ln -sf "$HOME/.config/sway/bin/sway-manager/SwayManager" "$HOME/.config/sway/bin/SwayManager"
    ln -sf "$HOME/.config/sway/bin/sway-manager/SwayManagerGUI" "$HOME/.config/sway/bin/SwayManagerGUI"
    ln -sf "$HOME/.config/sway/bin/sway-manager/SwayManagerGUI" "$HOME/.config/sway/bin/sway-manager-gui"
}

install_portal() {
    echo "Configurando XDG Desktop Portal..."
    if ! ~/.config/sway/bin/SwayManager portal install; then
        echo "Aviso: falha ao configurar portal (verifique o log acima)" >&2
    fi
}

install_udev() {
    if [ "$INSTALL_UDEV" != true ]; then
        return 0
    fi
    echo "Instalando regras udev para conservação de bateria (requer sudo)..."
    if [ -f ./udev/99-battery-conservation.rules ]; then
        sudo cp ./udev/99-battery-conservation.rules /etc/udev/rules.d/
        sudo udevadm control --reload-rules
        sudo udevadm trigger
        echo "Regras udev instaladas com sucesso!"
    fi
}

install_lswt() {
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
}

install_xdpw() {
    local XDPW_BIN=/usr/libexec/xdg-desktop-portal-wlr
    local NEEDS_XDPW_BUILD=false

    if [ ! -x "$XDPW_BIN" ]; then
        NEEDS_XDPW_BUILD=true
    elif ! strings "$XDPW_BIN" 2>/dev/null | grep -q "^Window: %s"; then
        NEEDS_XDPW_BUILD=true
    elif ! strings "$XDPW_BIN" 2>/dev/null | grep -q "cached_label"; then
        echo "  xdg-desktop-portal-wlr não possui o patch de label caching; reconstruindo..."
        NEEDS_XDPW_BUILD=true
    fi

    if [ "$NEEDS_XDPW_BUILD" = false ]; then
        echo "  xdg-desktop-portal-wlr já suporta captura de janela e tem o patch aplicado"
        return 0
    fi

    local TMP_XDPW
    TMP_XDPW=$(mktemp -d)
    echo "Clonando xdg-desktop-portal-wlr v0.8.4 em $TMP_XDPW..."
    git clone --branch v0.8.4 --depth 1 https://github.com/emersion/xdg-desktop-portal-wlr.git "$TMP_XDPW" 2>&1 | tail -1

    echo "Aplicando patch: cache de labels no chooser.c (corrige bug de tripla invocação)..."
    cp "$SCRIPT_DIR/installer/xdpw-chooser.c" "$TMP_XDPW/src/screencast/chooser.c"

    echo "Compilando e instalando xdg-desktop-portal-wlr v0.8.4 com patch..."
    meson setup "$TMP_XDPW-build" "$TMP_XDPW" --prefix=/usr -Dsd-bus-provider=libsystemd 2>&1 | tail -3
    ninja -C "$TMP_XDPW-build" 2>&1 | tail -3
    sudo ninja -C "$TMP_XDPW-build" install 2>&1 | tail -5
    rm -rf "$TMP_XDPW" "$TMP_XDPW-build"
    echo "  xdg-desktop-portal-wlr v0.8.4 (com patch) instalado com sucesso!"
}

install_polkit() {
    if [ "$INSTALL_POLKIT" != true ]; then
        return 0
    fi
    echo "Instalando agente Polkit nativo do SwayManager (substitui lxpolkit/lxqt/mate/gnome)..."
    local SWAY_BIN="$HOME/.config/sway/bin/SwayManager"
    if [ ! -x "$SWAY_BIN" ]; then
        echo "Erro: SwayManager não encontrado em $SWAY_BIN (a instalação acima falhou?)" >&2
        exit 1
    fi

    # 1. Desativa agentes rivais em execução (mesmos nomes do polkit-runner.sh).
    #    `pkill -f` (não `-x`): comm trunca em 15 chars (`lxqt-policykit-`),
    #    então match exato nunca casaria o lxqt.
    local RIVALS='polkit-gnome-authentication-agent-1|lxqt-policykit-agent|polkit-kde-authentication-agent-1|hyprpolkitagent|mate-polkit|lxpolkit'
    echo "  Parando agentes polkit rivais..."
    pkill -f -u "$USER" "$RIVALS" 2>/dev/null || true
    sleep 0.5
    # Rivais teimosos (respawn via systemd user): desabilita a unit legada.
    if [ -f "$HOME/.config/systemd/user/polkit-gnome-authentication-agent-1.service" ]; then
        echo "  Desabilitando unit legada polkit-gnome-authentication-agent-1.service..."
        systemctl --user disable --now polkit-gnome-authentication-agent-1.service 2>/dev/null || true
        rm -f "$HOME/.config/systemd/user/polkit-gnome-authentication-agent-1.service"
        systemctl --user daemon-reload 2>/dev/null || true
    fi
    if pgrep -f -u "$USER" "$RIVALS" 2>/dev/null; then
        echo "Aviso: ainda há agentes rivais rodando (PIDs acima); o nosso agente perderá o registro da sessão." >&2
    fi

    # 2. Autostart: troca o polkit-runner.sh pelo agente nativo no 30-autostart.
    #    (O daemon também spawna o agente residente como fallback — ver run_daemon.)
    local AUTOSTART="$HOME/.config/sway/config.d/30-autostart"
    if [ -f "$AUTOSTART" ] && grep -q "polkit-runner.sh" "$AUTOSTART"; then
        echo "  Atualizando autostart ($AUTOSTART)..."
        cp "$AUTOSTART" "$AUTOSTART.bak-polkit"
        sed -i 's|exec --no-startup-id ~/.config/sway/scripts/polkit-runner.sh &|exec $sway-manager polkit agent \&|' "$AUTOSTART"
        echo "  Backup do autostart anterior em $AUTOSTART.bak-polkit"
    elif [ -f "$AUTOSTART" ]; then
        echo "  Aviso: polkit-runner.sh não encontrado em $AUTOSTART; adicione manualmente: exec \$sway-manager polkit agent" >&2
    else
        echo "  Aviso: $AUTOSTART não existe (repo sway-config instalado?); pulando autostart." >&2
    fi

    # 3. Regra Sway p/ o diálogo (app_id org.swaymanager.PolkitAuth, specs/domains/polkit.md §3.4).
    local RULES="$HOME/.config/sway/config.d/99-rules"
    local RULE='for_window [app_id="org.swaymanager.PolkitAuth"] floating enable, move position center, sticky enable, focus'
    if [ -f "$RULES" ]; then
        if ! grep -q "org.swaymanager.PolkitAuth" "$RULES"; then
            echo "  Adicionando regra for_window do diálogo polkit ($RULES)..."
            printf '\n%s\n' "$RULE" >> "$RULES"
        else
            echo "  Regra for_window do diálogo polkit já presente."
        fi
        if command -v swaymsg &>/dev/null && [ -n "$SWAYSOCK" ]; then
            swaymsg reload 2>/dev/null || echo "  Aviso: 'swaymsg reload' falhou; recarregue o Sway manualmente." >&2
        fi
    else
        echo "  Aviso: $RULES não existe; adicione manualmente: $RULE" >&2
    fi

    # 4. Smoke test: sobe o agente, verifica o registro na sessão e derruba.
    echo "  Smoke test do agente..."
    "$SWAY_BIN" polkit agent > /tmp/sway-manager-polkit-install.log 2>&1 &
    local AGENT_PID=$!
    sleep 3
    if ! kill -0 "$AGENT_PID" 2>/dev/null; then
        echo "Erro: o agente morreu na inicialização. Log:" >&2
        cat /tmp/sway-manager-polkit-install.log >&2
        exit 1
    fi
    if ! busctl --system list 2>/dev/null | grep -q "sway-manager-polkit\|PolicyKit1"; then
        echo "Aviso: não foi possível confirmar o registro via busctl (verifique com 'SwayManager polkit status')." >&2
    fi
    kill "$AGENT_PID" 2>/dev/null || true
    wait "$AGENT_PID" 2>/dev/null || true
    echo "  Agente polkit nativo instalado! Recarregue o Sway para ativá-lo (o lxqt/mate continua instalado, mas desativado)."
}

main() {
    parse_args "$@"
    build_project
    install_binaries
    install_portal
    install_udev
    install_lswt
    install_xdpw
    install_polkit
    echo "Instalação concluída."
}

main "$@"
