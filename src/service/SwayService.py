import subprocess
import re
import os
from utils.exceptions import SwayException


class SwayService:
    @classmethod
    def get_monitores(cls) -> list[str]:
        cmd = 'swaymsg -t get_outputs | grep "\\"name\\":"'
        processo = subprocess.run(cmd, shell=True, text=True, capture_output=True)
        monitores = re.findall(r'"name":\s*"([^"]+)"', processo.stdout)

        return monitores

    @classmethod
    def validar_monitores(cls):
        if len(cls.get_monitores()) < 2:
            raise ValueError("Apenas um monitor encontrado!")

    @classmethod
    def recarregar(cls):
        subprocess.run(["swaymsg", "reload"])

    @classmethod
    def escrever_arquivo(
        cls, conteudo: str, arquivo: str, append=False, pasta="~/.config/sway/config.d/"
    ):
        pasta = os.path.expanduser(pasta)
        arquivo_config = os.path.join(pasta, arquivo)
        try:
            os.makedirs(os.path.dirname(arquivo_config), exist_ok=True)

            with open(arquivo_config, "w" if not append else "a") as f:
                f.write(conteudo)

            cls.recarregar()
        except Exception as e:
            raise SwayException(
                f"Não foi possivel escrever no arquivo {arquivo_config}: {e}"
            )
