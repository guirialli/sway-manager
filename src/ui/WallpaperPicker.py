from PySide6.QtWidgets import (
    QWidget,
    QApplication,
    QVBoxLayout,
    QListWidget,
    QListWidgetItem,
)
from PySide6.QtGui import QPixmap, QImage, QIcon
from PySide6.QtCore import QSize, Qt
from service.SwayService import SwayService
import ui.Styles as styles
from ui.components.ImageList import CarregadorDeImagens, ListaImagens


class WallpapaerPicker(QWidget):
    def __init__(self, pasta_imagem: str):
        super().__init__()
        self.pasta_imagens = pasta_imagem

        self.setWindowTitle("Seletor Wallpaper")
        self.resize(900, 600)

        self.setStyleSheet(f"""
            {styles.QWidget()}
            {styles.QListWidget()}
            {styles.QScrollbar()}
        """)

        QApplication.setDesktopFileName("sway.apps.wallpaper-picker")
        layout = QVBoxLayout()
        self.setLayout(layout)

        self.lista_imagens = ListaImagens(self.aplicar_wallpaper, lambda: self.close())
        self.lista_imagens.setViewMode(QListWidget.ViewMode.IconMode)
        self.lista_imagens.setIconSize(QSize(240, 135))
        self.lista_imagens.setResizeMode(QListWidget.ResizeMode.Adjust)
        self.lista_imagens.setSpacing(15)
        layout.addWidget(self.lista_imagens)

        self.lista_imagens.itemDoubleClicked.connect(self.aplicar_wallpaper)

        self.carregador = CarregadorDeImagens(self.pasta_imagens)
        self.carregador.imagem_carregada.connect(self.carregar_imagem)
        self.carregador.start()

    def carregar_imagem(self, caminho: str, image: QImage):
        item = QListWidgetItem()
        pixmap = QPixmap.fromImage(image)
        item.setIcon(QIcon(pixmap))
        item.setData(Qt.ItemDataRole.UserRole, caminho)

        self.lista_imagens.addItem(item)

    def aplicar_wallpaper(self, item: QListWidgetItem):
        caminho_imagem = item.data(Qt.ItemDataRole.UserRole)
        conteudo = f'output "*" bg {caminho_imagem} fill\n'

        try:
            SwayService.escrever_arquivo(arquivo="42-wallpaper", conteudo=conteudo)
            self.close()
        except Exception as e:
            print(f"Erro ao aplicar wallpaper {e}")
