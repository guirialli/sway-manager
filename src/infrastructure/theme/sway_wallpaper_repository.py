import os
import re
from typing import Optional
from domain.theme.repositories import IWallpaperRepository
from utils.exceptions import SwayException


class SwayWallpaperRepository(IWallpaperRepository):
    def _escrever_arquivo(
        self, conteudo: str, arquivo: str, append=False, pasta="~/.config/sway/config.d/"
    ):
        pasta = os.path.expanduser(pasta)
        arquivo_config = os.path.join(pasta, arquivo)
        try:
            os.makedirs(os.path.dirname(arquivo_config), exist_ok=True)

            with open(arquivo_config, "w" if not append else "a") as f:
                f.write(conteudo)

            import subprocess
            subprocess.run(["swaymsg", "reload"])
        except Exception as e:
            raise SwayException(
                f"Não foi possivel escrever no arquivo {arquivo_config}: {e}"
            )

    def set_wallpaper(self, image_path: str) -> None:
        if not os.path.isfile(image_path):
            raise SwayException(f"Arquivo de imagem não encontrado: {image_path}")

        conteudo = f'output "*" bg "{image_path}" fill\n'
        self._escrever_arquivo(arquivo="42-wallpaper", conteudo=conteudo)

        sway_wp_link = os.path.expanduser("~/.config/sway/wallpaper")
        try:
            sway_dir = os.path.dirname(sway_wp_link)
            os.makedirs(sway_dir, exist_ok=True)
            if os.path.islink(sway_wp_link) or os.path.exists(sway_wp_link):
                os.remove(sway_wp_link)
            os.symlink(image_path, sway_wp_link)
        except Exception as e:
            print(f"Aviso ao atualizar symlink ~/.config/sway/wallpaper: {e}")

    def get_current_wallpaper(self) -> Optional[str]:
        current_wp = os.path.expanduser("~/.config/sway/wallpaper")
        if os.path.islink(current_wp):
            try:
                target = os.readlink(current_wp)
                if os.path.isfile(target):
                    return target
            except Exception:
                pass
        elif os.path.isfile(current_wp):
            return current_wp

        wp_config_file = os.path.expanduser("~/.config/sway/config.d/42-wallpaper")
        if os.path.isfile(wp_config_file):
            try:
                with open(wp_config_file, "r") as f:
                    content = f.read()
                match = re.search(r'bg\s+(?:"([^"]+)"|\'([^\']+)\'|(\S+))', content)
                if match:
                    path = match.group(1) or match.group(2) or match.group(3)
                    if path and os.path.isfile(path):
                        return path
            except Exception as e:
                print(f"Erro ao ler 42-wallpaper: {e}")

        config_d = os.path.expanduser("~/.config/sway/config.d")
        if os.path.isdir(config_d):
            for file_name in sorted(os.listdir(config_d)):
                file_path = os.path.join(config_d, file_name)
                if os.path.isfile(file_path):
                    try:
                        with open(file_path, "r") as f:
                            content = f.read()
                        match = re.search(r'bg\s+(?:"([^"]+)"|\'([^\']+)\'|(\S+))', content)
                        if match:
                            path = match.group(1) or match.group(2) or match.group(3)
                            if path and os.path.isfile(path):
                                return path
                    except Exception:
                        pass

        return None
