from PySide6.QtCore import QThread, Signal, Qt
from PySide6.QtGui import QKeyEvent, QImage
from PySide6.QtWidgets import QListWidget
import os


class CarregadorDeImagens(QThread):
    imagem_carregada = Signal(str, QImage)

    def __init__(self, pasta: str):
        super().__init__()
        self.pasta = pasta

    def run(self, /) -> None:
        extensoes_validar = (".jpg", ".jpeg", ".png", ".webp")
        if not os.path.isdir(self.pasta):
            return
        for nome_arquivo in sorted(os.listdir(self.pasta)):
            if not nome_arquivo.lower().endswith(extensoes_validar):
                continue

            caminho_completo = os.path.join(self.pasta, nome_arquivo)
            imagem = QImage(caminho_completo)

            if imagem.isNull():
                continue

            miniatura = imagem.scaled(
                240,
                135,
                Qt.AspectRatioMode.KeepAspectRatioByExpanding,
                Qt.TransformationMode.SmoothTransformation,
            )
            self.imagem_carregada.emit(caminho_completo, miniatura)


class ListaImagens(QListWidget):
    def __init__(self, fn_aplicar_image, fn_close):
        super().__init__()
        self.fn_aplicar_image = fn_aplicar_image
        self.fn_close = fn_close

    def keyPressEvent(self, event: QKeyEvent) -> None:
        if (
            event.key() in (Qt.Key.Key_Enter, Qt.Key.Key_Return)
            and self.fn_aplicar_image
        ):
            item = self.currentItem()
            self.fn_aplicar_image(item)
        elif event.key() == Qt.Key.Key_Escape and self.fn_close:
            self.fn_close()
        else:
            return super().keyPressEvent(event)
