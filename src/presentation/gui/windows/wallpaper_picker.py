from PySide6.QtWidgets import (
    QWidget,
    QApplication,
    QVBoxLayout,
    QListWidget,
    QListWidgetItem,
    QLabel,
    QHBoxLayout,
)
from PySide6.QtGui import QPixmap, QImage, QIcon
from PySide6.QtCore import QSize, Qt
from infrastructure.theme.sway_wallpaper_repository import SwayWallpaperRepository
from application.theme.set_wallpaper_use_case import SetWallpaperUseCase
from infrastructure.theme.gtk_qt_theme_repository import GtkQtThemeRepository
import os
import presentation.gui.styles as styles
from presentation.gui.components.image_list import CarregadorDeImagens, ListaImagens
from utils.string import StringUtils


class WallpaperPickerWindow(QWidget):
    def __init__(self, pasta_imagem: str | None = None):
        super().__init__()
        self.use_case = SetWallpaperUseCase(SwayWallpaperRepository())
        self.pasta_imagens = pasta_imagem or self.use_case.get_wallpaper_folder()
        self.theme_repo = GtkQtThemeRepository()
        self.mode = self.theme_repo.get_state().current_theme

        self.setWindowTitle("Seletor de Papel de Parede")
        self.resize(880, 560)
        self.setObjectName("mainWindow")

        self.setStyleSheet(styles.get_stylesheet(self.mode))

        QApplication.setDesktopFileName("sway.apps.wallpaper-picker")
        layout = QVBoxLayout()
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(15)
        self.setLayout(layout)

        header_layout = QHBoxLayout()
        title_lbl = QLabel("🖼️ Galeria de Wallpapers")
        title_lbl.setStyleSheet(f"font-size: 18px; font-weight: bold; font-family: {styles.FONT_FAMILY};")
        header_layout.addWidget(title_lbl)
        header_layout.addStretch()
        layout.addLayout(header_layout)

        self.lista_imagens = ListaImagens(self.aplicar_wallpaper, lambda: self.close())
        self.lista_imagens.setViewMode(QListWidget.ViewMode.IconMode)
        self.lista_imagens.setIconSize(QSize(240, 135))
        self.lista_imagens.setResizeMode(QListWidget.ResizeMode.Adjust)
        self.lista_imagens.setMovement(QListWidget.Movement.Static)
        self.lista_imagens.setSpacing(16)
        layout.addWidget(self.lista_imagens)

        self.lista_imagens.itemDoubleClicked.connect(self.aplicar_wallpaper)

        self.carregador = CarregadorDeImagens(self.pasta_imagens)
        self.carregador.imagem_carregada.connect(self.carregar_imagem)
        self.carregador.start()

    def carregar_imagem(self, caminho: str, image: QImage):
        item = QListWidgetItem()
        pixmap = QPixmap.fromImage(image)
        item.setIcon(QIcon(pixmap))
        item.setSizeHint(QSize(250, 160))
        
        nome_completo = os.path.basename(caminho)
        item.setToolTip(nome_completo)
        item.setText(StringUtils.truncar_nome_arquivo(nome_completo, max_len=20))
        item.setData(Qt.ItemDataRole.UserRole, caminho)

        self.lista_imagens.addItem(item)

    def aplicar_wallpaper(self, item: QListWidgetItem):
        caminho_imagem = item.data(Qt.ItemDataRole.UserRole)
        if not caminho_imagem:
            return

        try:
            self.use_case.execute(caminho_imagem)
            self.close()
        except Exception as e:
            print(f"Erro ao aplicar wallpaper {e}")

    def closeEvent(self, event):
        if hasattr(self, "carregador") and self.carregador and self.carregador.isRunning():
            self.carregador.requestInterruption()
            self.carregador.quit()
            self.carregador.wait(1000)
        if hasattr(self, "lista_imagens") and self.lista_imagens:
            self.lista_imagens.clear()
        super().closeEvent(event)
