import sys
import threading
import traceback


def setup_exception_handlers(logger=None):
    """
    Configura manipuladores globais de exceção para interceptar falhas
    não tratadas na thread principal e em threads secundárias do Python/Qt,
    registrando o traceback completo no logger sem derrubar o daemon.
    """

    def handle_sys_exception(exc_type, exc_value, exc_traceback):
        if issubclass(exc_type, KeyboardInterrupt):
            sys.__excepthook__(exc_type, exc_value, exc_traceback)
            return
        formatted_tb = "".join(
            traceback.format_exception(exc_type, exc_value, exc_traceback)
        )
        if logger:
            logger.error(
                f"⚠️ Exceção não tratada capturada no daemon:\n{formatted_tb.strip()}"
            )
        else:
            print(f"⚠️ Exceção não tratada: {formatted_tb.strip()}", file=sys.stderr)

    def handle_thread_exception(args):
        if issubclass(args.exc_type, KeyboardInterrupt):
            return
        formatted_tb = "".join(
            traceback.format_exception(args.exc_type, args.exc_value, args.exc_traceback)
        )
        thread_name = args.thread.name if args.thread else "desconhecida"
        if logger:
            logger.error(
                f"⚠️ Exceção não tratada em thread secundária ('{thread_name}'):\n{formatted_tb.strip()}"
            )
        else:
            print(
                f"⚠️ Exceção não tratada na thread '{thread_name}': {formatted_tb.strip()}",
                file=sys.stderr,
            )

    sys.excepthook = handle_sys_exception
    threading.excepthook = handle_thread_exception
