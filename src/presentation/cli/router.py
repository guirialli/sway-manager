import sys


def show_help():
    help_text = """
SwayManager - Suite de Gerenciamento para Sway e SwayFX

Uso:
  SwayManager <comando> [opções]

Comandos Disponíveis:
  daemon                   Inicia o servidor daemon persistente em background (binário Rust).
  daemon log [-f|--follow] Exibe ou acompanha em tempo real os logs do dia (~/.config/sway-manager/logs/).
  settings, config         Abre o painel gráfico do Control Center (Configurações).
  monitor                  Abre a janela gráfica para alternar layouts de monitores.
  wallpaper [pasta]        Abre a janela gráfica para selecionar papel de parede.
  osd brilho [up|down]     Ajusta o brilho e exibe o OSD gráfico.
  osd volume [up|down|mute] Ajusta o volume e exibe o OSD gráfico.
  battery [toggle|status]  Alterna a conservação de bateria (~80% vs 100%) ou retorna o status (Waybar JSON).
  idle [toggle|status] [flag] Alterna o inibidor de suspensão swayidle (-s/-n/-r) ou retorna o status (Waybar JSON).
  theme [toggle|status]    Alterna o tema (Dark/Light) do GTK, Qt e Foot ou retorna o status (Waybar JSON).
  power [toggle|status] [flag] Alterna o perfil de energia (-p/-b/-s) ou retorna o status (Waybar JSON).
  screenshot [full|area|window] Tira uma captura de tela e copia para a área de transferência.
  portal                   Abre o seletor de compartilhamento de tela para o xdg-desktop-portal-wlr.
  portal status            Verifica se o compartilhamento de tela está disponível.
  portal test              Abre o seletor em modo de teste e exibe o resultado.
  menu [categoria]         Abre o lançador de aplicativos Wofi customizado.
  clipboard [clear|pin]    Abre o gerenciador de clipboard Wofi com suporte a miniaturas e favoritos.
  lock                     Bloqueia a tela usando swaylock customizado com wallpaper e tema.
  -h, --help               Exibe esta mensagem de ajuda.

O daemon e o roteamento IPC vivem no binário Rust (`daemon/`); este
módulo Python executa os comandos localmente (modo GUI/dev).
"""
    print(help_text.strip())


def handle_daemon_log(args: list[str]):
    import os
    import time
    from datetime import datetime

    log_path = os.path.expanduser(
        f"~/.config/sway-manager/logs/log-{datetime.now().strftime('%Y-%m-%d')}.txt"
    )
    if not os.path.exists(log_path):
        print(f"Nenhum log registrado para hoje em: {log_path}")
        return

    follow = any(flag in args for flag in ("-f", "--follow"))

    try:
        with open(log_path, "r", encoding="utf-8") as f:
            lines = f.readlines()
            if not follow:
                tail_lines = lines[-50:] if len(lines) > 50 else lines
                print("".join(tail_lines).strip())
                return

            print("".join(lines).strip())
            print(f"--- Acompanhando logs em tempo real ({log_path}) [Ctrl+C para sair] ---")
            f.seek(0, os.SEEK_END)
            while True:
                line = f.readline()
                if line:
                    print(line, end="")
                else:
                    time.sleep(0.2)
    except KeyboardInterrupt:
        print("\nMonitoramento de logs encerrado.")
    except Exception as ex:
        print(f"Erro ao ler arquivo de log: {ex}")


def dispatch_cli_command(args: list[str]):
    if not args or len(args) < 2:
        return
    cmd = str(args[1]).lower()
    from presentation.cli.handlers import CLIHandlers

    if cmd in ("settings", "config", "config-center"):
        CLIHandlers.handle_settings()
    elif cmd == "monitor":
        CLIHandlers.handle_monitor(args)
    elif cmd == "wallpaper":
        CLIHandlers.handle_wallpaper(args)
    elif cmd == "osd":
        CLIHandlers.handle_osd(args)
    elif cmd in ("brilho", "brightness"):
        CLIHandlers.handle_brightness(args)
    elif cmd == "battery":
        CLIHandlers.handle_battery(args)
    elif cmd == "idle":
        CLIHandlers.handle_idle(args)
    elif cmd == "theme":
        CLIHandlers.handle_theme(args)
    elif cmd == "power":
        CLIHandlers.handle_power(args)
    elif cmd == "screenshot":
        CLIHandlers.handle_screenshot(args)
    elif cmd == "menu":
        CLIHandlers.handle_menu(args)
    elif cmd in ("clipboard", "clip"):
        CLIHandlers.handle_clipboard(args)
    elif cmd == "lock":
        CLIHandlers.handle_lock(args)
    elif cmd == "portal":
        CLIHandlers.handle_portal(args)
    else:
        print(f"Comando '{cmd}' não reconhecido pelo SwayManager.\n", file=sys.stderr)


def run_cli():
    args = list(sys.argv)

    if "--standalone" in args:
        args.remove("--standalone")
        dispatch_cli_command(args)
        return

    if len(args) == 1:
        show_help()
        return

    app = str(args[1]).lower()

    if app in ("-h", "--help", "help"):
        show_help()
        return

    # O daemon agora é o binário Rust (`SwayManager daemon`, instalado em
    # ~/.config/sway/bin/): este entrypoint Python só executa localmente.
    # `daemon log` continua disponível aqui para leitura dos logs do dia.
    if app in ("daemon", "--daemon", "-d"):
        if len(args) > 2 and str(args[2]).lower() in ("log", "logs", "-l"):
            handle_daemon_log(args)
            return

        print(
            "O daemon SwayManager é o binário Rust instalado em ~/.config/sway/bin/SwayManager.\n"
            "Use 'SwayManager daemon' (binário) para iniciá-lo; este entrypoint Python executa apenas localmente.",
            file=sys.stderr,
        )
        sys.exit(1)

    # Portal is invoked by xdg-desktop-portal-wlr as a subprocess; it must run
    # locally regardless of whether the daemon is active.
    if app == "portal":
        dispatch_cli_command(args)
        return

    # Sem cliente IPC em Python: o binário Rust responde pelo socket.
    # Aqui, executa localmente (comportamento do antigo `--standalone`).
    dispatch_cli_command(args)
