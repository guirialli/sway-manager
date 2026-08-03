import sys


def show_help():
    help_text = """
SwayManager - Suite de Gerenciamento para Sway e SwayFX

Uso:
  SwayManager <comando> [opções]

Comandos Disponíveis:
  daemon                   Inicia o servidor daemon persistente em background.
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
  menu [categoria]         Abre o lançador de aplicativos Wofi customizado.
  clipboard [clear|pin]    Abre o gerenciador de clipboard Wofi com suporte a miniaturas e favoritos.
  lock                     Bloqueia a tela usando swaylock customizado com wallpaper e tema.
  -h, --help               Exibe esta mensagem de ajuda.
"""
    print(help_text.strip())


def handle_daemon_log(args: list[str]):
    import os
    import time
    from infrastructure.logging.async_logger import get_logger

    log_path = get_logger().get_today_log_path()
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


def run_cli():
    args = sys.argv

    if len(args) == 1:
        show_help()
        return

    app = str(args[1]).lower()

    if app in ("-h", "--help", "help"):
        show_help()
        return

    if app in ("daemon", "--daemon", "-d"):
        if len(args) > 2 and str(args[2]).lower() in ("log", "logs", "-l"):
            handle_daemon_log(args)
            return
        from infrastructure.daemon.daemon_server import SwayManagerDaemon
        daemon = SwayManagerDaemon()
        daemon.start()
        return

    from infrastructure.daemon.daemon_client import SwayManagerClient
    if not SwayManagerClient.send_command(args):
        print(
            "❌ Erro: O SwayManager Daemon não está em execução.\n"
            "Inicie o serviço com 'SwayManager daemon' ou verifique a inicialização do Sway.",
            file=sys.stderr,
        )
        sys.exit(1)
