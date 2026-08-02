import os
import sys
import json
import html
import shutil
import tempfile
import subprocess
from typing import List, Optional, Dict
from domain.clipboard.entities import ClipboardItem
from domain.clipboard.repositories import IClipboardRepository
from infrastructure.menu.icon_resolver import IconResolver

try:
    from PySide6.QtGui import QImage
    from PySide6.QtCore import Qt
    PYSIDE_AVAILABLE = True
except ImportError:
    PYSIDE_AVAILABLE = False


class CliphistRepository(IClipboardRepository):
    def __init__(
        self,
        fav_config_path: Optional[str] = None,
        cache_dir: Optional[str] = None,
    ):
        self.fav_config_path = os.path.expanduser(
            fav_config_path or "~/.config/sway-manager/clipboard_favorites.json"
        )
        self.fav_media_dir = os.path.expanduser(
            "~/.config/sway-manager/clipboard_favs"
        )
        self.cache_dir = os.path.expanduser(
            cache_dir or "~/.cache/sway-manager/clipboard_thumbs"
        )
        os.makedirs(os.path.dirname(self.fav_config_path), exist_ok=True)
        os.makedirs(self.fav_media_dir, exist_ok=True)
        os.makedirs(self.cache_dir, exist_ok=True)

    def ensure_daemon_running(self) -> bool:
        """Garante que a captura de clipboard 'wl-paste --watch cliphist store' esteja ativa."""
        if not shutil.which("cliphist") or not shutil.which("wl-paste"):
            print("Aviso: 'cliphist' ou 'wl-paste' não estão instalados no sistema.")
            return False

        try:
            res = subprocess.run(
                ["pgrep", "-f", "cliphist store"],
                capture_output=True,
                text=True,
            )
            if res.returncode == 0 and res.stdout.strip():
                return True

            subprocess.Popen(
                ["wl-paste", "--watch", "cliphist", "store"],
                start_new_session=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            return True
        except Exception as e:
            print(f"Erro ao verificar/iniciar daemon do cliphist: {e}")
            return False

    def get_favorites(self) -> List[ClipboardItem]:
        if not os.path.isfile(self.fav_config_path):
            return []

        try:
            with open(self.fav_config_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                items = []
                for entry in data:
                    items.append(
                        ClipboardItem(
                            id=entry["id"],
                            text=entry["text"],
                            raw_preview=entry.get("raw_preview", entry["text"]),
                            is_image=entry.get("is_image", False),
                            image_path=entry.get("image_path"),
                            is_favorite=True,
                        )
                    )
                return items
        except Exception as e:
            print(f"Erro ao carregar favoritos: {e}")
            return []

    def save_favorites(self, favorites: List[ClipboardItem]) -> bool:
        try:
            serialized = []
            for fav in favorites:
                serialized.append(
                    {
                        "id": fav.id,
                        "text": fav.text,
                        "raw_preview": fav.raw_preview,
                        "is_image": fav.is_image,
                        "image_path": fav.image_path,
                    }
                )
            with open(self.fav_config_path, "w", encoding="utf-8") as f:
                json.dump(serialized, f, indent=2, ensure_ascii=False)
                f.write("\n")
            return True
        except Exception as e:
            print(f"Erro ao salvar favoritos: {e}")
            return False

    def add_favorite(self, item: ClipboardItem) -> bool:
        favorites = self.get_favorites()
        if any(f.id == item.id or (not f.is_image and f.text == item.text) for f in favorites):
            return True

        fav_id = f"fav_{len(favorites) + 1}_{hash(item.raw_preview) & 0xffffff}"
        image_path = None

        if item.is_image:
            # Salva cópia permanente da imagem do favorito
            fav_img_file = os.path.join(self.fav_media_dir, f"{fav_id}.png")
            if item.image_path and os.path.exists(item.image_path):
                shutil.copy(item.image_path, fav_img_file)
                image_path = fav_img_file
            else:
                raw_bytes = self._decode_cliphist_item(item.id)
                if raw_bytes:
                    with open(fav_img_file, "wb") as img_out:
                        img_out.write(raw_bytes)
                    image_path = fav_img_file
        else:
            raw_bytes = self._decode_cliphist_item(item.id)
            if raw_bytes:
                # Se era item de histórico, persistir texto no arquivo
                fav_txt_file = os.path.join(self.fav_media_dir, f"{fav_id}.txt")
                with open(fav_txt_file, "wb") as txt_out:
                    txt_out.write(raw_bytes)

        new_fav = ClipboardItem(
            id=fav_id,
            text=item.text,
            raw_preview=item.raw_preview,
            is_image=item.is_image,
            image_path=image_path,
            is_favorite=True,
        )
        favorites.append(new_fav)
        self.save_favorites(favorites)
        self._notify("Clipboard", f"Item fixado nos favoritos!")
        return True

    def remove_favorite(self, item_id: str) -> bool:
        favorites = self.get_favorites()
        updated = [f for f in favorites if f.id != item_id and f.text != item_id]
        self.save_favorites(updated)
        self._notify("Clipboard", "Item removido dos favoritos.")
        return True

    def get_clipboard_items(self) -> List[ClipboardItem]:
        items: List[ClipboardItem] = []

        # 1. Adicionar Ações Especiais
        items.append(
            ClipboardItem(
                id="action_clear",
                text="🧹 [Limpar Histórico do Clipboard]",
                raw_preview="🧹 Limpar Histórico do Clipboard",
                is_action=True,
                action_type="clear",
            )
        )
        items.append(
            ClipboardItem(
                id="action_manage_favs",
                text="📌 [Gerenciar / Fixar Favoritos]",
                raw_preview="📌 Gerenciar / Fixar Favoritos",
                is_action=True,
                action_type="manage_favorites",
            )
        )

        # 2. Adicionar Favoritos (Fixados)
        favorites = self.get_favorites()
        items.extend(favorites)

        # 3. Carregar Histórico do Cliphist
        if not shutil.which("cliphist"):
            return items

        try:
            res = subprocess.run(
                ["cliphist", "list"],
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            if res.returncode == 0 and res.stdout:
                lines = res.stdout.strip().split("\n")
                fav_texts = {f.text for f in favorites}

                for line in lines:
                    if not line.strip():
                        continue
                    parts = line.split("\t", 1)
                    item_id = parts[0].strip()
                    summary = parts[1] if len(parts) > 1 else ""

                    if summary in fav_texts:
                        continue  # Já listado em favoritos

                    is_img = False
                    img_path = None

                    # Verificar se é binário / imagem
                    if summary.strip().startswith("[[ binary data"):
                        is_img = True
                        img_path = self._get_or_create_thumbnail(item_id)
                        display_text = f"[Imagem] {summary.strip()}"
                    else:
                        display_text = summary

                    items.append(
                        ClipboardItem(
                            id=item_id,
                            text=display_text,
                            raw_preview=summary,
                            is_image=is_img,
                            image_path=img_path,
                            is_favorite=False,
                        )
                    )
        except Exception as e:
            print(f"Erro ao ler histórico do cliphist: {e}")

        return items

    def _get_or_create_thumbnail(self, item_id: str) -> Optional[str]:
        thumb_path = os.path.join(self.cache_dir, f"clip_{item_id}.png")
        if os.path.exists(thumb_path):
            return thumb_path

        raw_bytes = self._decode_cliphist_item(item_id)
        if not raw_bytes:
            return None

        if PYSIDE_AVAILABLE:
            try:
                qimg = QImage.fromData(raw_bytes)
                if not qimg.isNull():
                    scaled = qimg.scaled(
                        64,
                        64,
                        Qt.AspectRatioMode.KeepAspectRatio,
                        Qt.TransformationMode.SmoothTransformation,
                    )
                    scaled.save(thumb_path, "PNG")
                    return thumb_path
            except Exception as e:
                print(f"Erro ao processar miniatura da imagem {item_id}: {e}")

        # Se falhar a miniatura ou PySide6 não disponível, salvar os bytes crus como arquivo temporário
        try:
            with open(thumb_path, "wb") as f:
                f.write(raw_bytes)
            return thumb_path
        except Exception:
            return None

    def _decode_cliphist_item(self, item_id: str) -> Optional[bytes]:
        try:
            res = subprocess.run(
                ["cliphist", "decode", str(item_id)],
                capture_output=True,
            )
            if res.returncode == 0:
                return res.stdout
        except Exception as e:
            print(f"Erro ao decodificar item {item_id} do cliphist: {e}")
        return None

    def copy_to_clipboard(self, item: ClipboardItem) -> bool:
        if not shutil.which("wl-copy"):
            print("Erro: 'wl-copy' não encontrado.")
            return False

        try:
            if item.is_favorite:
                # Favorito pode ser imagem ou texto salvo
                fav_txt_file = os.path.join(self.fav_media_dir, f"{item.id}.txt")
                fav_img_file = os.path.join(self.fav_media_dir, f"{item.id}.png")

                if item.is_image and (
                    (item.image_path and os.path.exists(item.image_path))
                    or os.path.exists(fav_img_file)
                ):
                    path_to_use = item.image_path if (item.image_path and os.path.exists(item.image_path)) else fav_img_file
                    with open(path_to_use, "rb") as f:
                        data = f.read()
                    subprocess.run(["wl-copy"], input=data, check=True)
                elif os.path.exists(fav_txt_file):
                    with open(fav_txt_file, "rb") as f:
                        data = f.read()
                    subprocess.run(["wl-copy"], input=data, check=True)
                else:
                    subprocess.run(["wl-copy"], input=item.text.encode("utf-8"), check=True)
            else:
                raw_bytes = self._decode_cliphist_item(item.id)
                if raw_bytes:
                    subprocess.run(["wl-copy"], input=raw_bytes, check=True)
                else:
                    subprocess.run(["wl-copy"], input=item.raw_preview.encode("utf-8"), check=True)

            self._notify("Clipboard", "Copiado para a área de transferência!")
            return True
        except Exception as e:
            print(f"Erro ao copiar para a área de transferência: {e}")
            return False

    def clear_history(self, keep_favorites: bool = True) -> bool:
        try:
            if shutil.which("cliphist"):
                subprocess.run(["cliphist", "wipe"], check=True)

            # Limpa miniaturas do cache
            if os.path.exists(self.cache_dir):
                for fname in os.listdir(self.cache_dir):
                    fpath = os.path.join(self.cache_dir, fname)
                    if os.path.isfile(fpath):
                        try:
                            os.remove(fpath)
                        except Exception:
                            pass

            self._notify("Clipboard", "Histórico do clipboard limpo com sucesso!")
            return True
        except Exception as e:
            print(f"Erro ao limpar histórico do clipboard: {e}")
            return False

    def launch_menu(
        self, items: List[ClipboardItem], prompt: str = "Clipboard..."
    ) -> Optional[ClipboardItem]:
        if not items:
            return None

        line_to_item: Dict[str, ClipboardItem] = {}
        input_lines: List[str] = []

        default_copy_icon = IconResolver.resolve_icon_path("edit-copy")
        default_clear_icon = IconResolver.resolve_icon_path("edit-clear")
        default_fav_icon = IconResolver.resolve_icon_path("emblem-favorite")

        for item in items:
            escaped_text = html.escape(item.text)

            if item.is_action:
                icon_path = (
                    default_clear_icon
                    if item.action_type == "clear"
                    else default_fav_icon
                )
                line_key = f"img:{icon_path}:text:<b>{escaped_text}</b>"
            elif item.is_favorite:
                icon_path = item.image_path or default_fav_icon
                line_key = f"img:{icon_path}:text:📌 <b>[Favorito]</b> {escaped_text}"
            else:
                icon_path = item.image_path or default_copy_icon
                line_key = f"img:{icon_path}:text:{escaped_text}"

            input_lines.append(line_key)
            line_to_item[line_key] = item

        input_payload = "\n".join(input_lines)

        wofi_cmd = [
            "wofi",
            "--dmenu",
            "--allow-images",
            "--allow-markup",
            "--insensitive",
            f"--prompt={prompt}",
        ]

        home_config = os.path.expanduser("~/.config/wofi/config")
        home_style = os.path.expanduser("~/.config/wofi/style.css")
        if os.path.exists(home_config):
            wofi_cmd.extend(["--conf", home_config])
        if os.path.exists(home_style):
            wofi_cmd.extend(["--style", home_style])

        try:
            process = subprocess.Popen(
                wofi_cmd,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            stdout, _ = process.communicate(input=input_payload)

            if process.returncode != 0 or not stdout:
                return None

            selected_line = stdout.strip()
            return line_to_item.get(selected_line)
        except Exception as ex:
            print(f"Erro ao executar Wofi para o clipboard: {ex}")
            return None

    def _notify(self, title: str, message: str) -> None:
        try:
            if shutil.which("notify-send"):
                subprocess.Popen(["notify-send", "-a", "SwayManager", title, message])
        except Exception:
            pass
