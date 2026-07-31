import os
import subprocess
from PySide6.QtCore import Qt, QSize
from PySide6.QtWidgets import (
    QApplication,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QTabWidget,
    QFileDialog,
    QListWidget,
    QListWidgetItem,
    QGroupBox,
)
from PySide6.QtGui import QIcon, QPixmap, QImage
import ui.Styles as styles
from ui.components.ImageList import CarregadorDeImagens, ListaImagens
from service.BatteryService import BatteryService
from service.IdleService import IdleService
from service.ThemeService import ThemeService
from service.PowerProfileService import PowerProfileService
from service.SwayService import SwayService


class ConfigCenterWindow(QWidget):
    def __init__(self):
        super().__init__()
        QApplication.setDesktopFileName("sway.apps.config-center")
        self.setWindowTitle("SwayManager Control Center")
        self.resize(920, 640)

        self.pasta_imagens = os.path.expanduser("~/Imagens/Wallpapers")
        if not os.path.exists(self.pasta_imagens):
            self.pasta_imagens = os.path.expanduser("~/Pictures")

        self.carregador = None
        self.setup_ui()
        self.apply_styles()

    def setup_ui(self):
        main_layout = QVBoxLayout()
        main_layout.setContentsMargins(15, 15, 15, 15)
        main_layout.setSpacing(12)

        # Header Title Bar
        header_layout = QHBoxLayout()
        header_title = QLabel("⚙️ SwayManager Control Center")
        header_title.setStyleSheet(
            "font-size: 22px; font-weight: bold; color: #9d7cd8; background: transparent;"
        )
        header_layout.addWidget(header_title)
        header_layout.addStretch()

        main_layout.addLayout(header_layout)

        # Tab Widget
        self.tabs = QTabWidget()
        self.tabs.addTab(self.create_wallpaper_tab(), "🖼️ Papel de Parede")
        self.tabs.addTab(self.create_theme_tab(), "🌓 Aparência & Tema")
        self.tabs.addTab(self.create_power_tab(), "🔋 Bateria & Energia")
        self.tabs.addTab(self.create_idle_tab(), "☕ Suspensão & Idle")

        main_layout.addWidget(self.tabs)
        self.setLayout(main_layout)

    def create_wallpaper_tab(self) -> QWidget:
        widget = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(10, 10, 10, 10)
        layout.setSpacing(10)

        # Top Bar: Directory display + Folder selector button
        top_bar = QHBoxLayout()
        self.lbl_pasta = QLabel(f"Pasta: {self.pasta_imagens}")
        self.lbl_pasta.setStyleSheet("font-size: 13px; color: #a9b1d6; background: transparent;")

        btn_select = QPushButton("📁 Escolher Pasta")
        btn_select.clicked.connect(self.selecionar_pasta)

        top_bar.addWidget(self.lbl_pasta)
        top_bar.addStretch()
        top_bar.addWidget(btn_select)
        layout.addLayout(top_bar)

        # Lista de Imagens - Idêntica ao WallpaperPicker ($mod+SHIFT+t)
        self.lista_imagens = ListaImagens(self.aplicar_wallpaper, None)
        self.lista_imagens.setViewMode(QListWidget.ViewMode.IconMode)
        self.lista_imagens.setIconSize(QSize(240, 135))
        self.lista_imagens.setResizeMode(QListWidget.ResizeMode.Adjust)
        self.lista_imagens.setSpacing(15)
        self.lista_imagens.itemDoubleClicked.connect(self.aplicar_wallpaper)

        layout.addWidget(self.lista_imagens)

        # Bottom Bar: Action buttons
        bottom_bar = QHBoxLayout()
        btn_aplicar = QPushButton("✨ Aplicar Papel de Parede")
        btn_aplicar.clicked.connect(self.aplicar_wallpaper_selecionado)
        bottom_bar.addStretch()
        bottom_bar.addWidget(btn_aplicar)

        layout.addLayout(bottom_bar)
        widget.setLayout(layout)

        self.iniciar_carregador_imagens()
        return widget

    def selecionar_pasta(self):
        nova_pasta = QFileDialog.getExistingDirectory(
            self, "Selecionar Pasta de Wallpapers", self.pasta_imagens
        )
        if nova_pasta:
            self.pasta_imagens = nova_pasta
            self.lbl_pasta.setText(f"Pasta: {self.pasta_imagens}")
            self.iniciar_carregador_imagens()

    def iniciar_carregador_imagens(self):
        self.lista_imagens.clear()
        if self.carregador and self.carregador.isRunning():
            self.carregador.terminate()

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
        if not item:
            return
        caminho_imagem = item.data(Qt.ItemDataRole.UserRole)
        if not caminho_imagem:
            return

        conteudo = f'output "*" bg {caminho_imagem} fill\n'
        try:
            SwayService.escrever_arquivo(arquivo="42-wallpaper", conteudo=conteudo)
            subprocess.run(
                ["notify-send", "Papel de Parede", f"Aplicado: {os.path.basename(caminho_imagem)}"]
            )
        except Exception as e:
            print(f"Erro ao aplicar wallpaper: {e}")

    def aplicar_wallpaper_selecionado(self):
        item = self.lista_imagens.currentItem()
        if item:
            self.aplicar_wallpaper(item)

    def create_theme_tab(self) -> QWidget:
        widget = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(15)

        group = QGroupBox("Alternar Tema GTK & Terminal")
        group_layout = QVBoxLayout()

        self.lbl_current_theme = QLabel(
            f"Tema Atual: {ThemeService.get_current_theme().upper()}"
        )
        self.lbl_current_theme.setStyleSheet(
            "font-size: 16px; font-weight: bold; color: #7aa2f7; background: transparent;"
        )

        btn_toggle_theme = QPushButton("🌓 Alternar Tema (Dark / Light)")
        btn_toggle_theme.clicked.connect(self.toggle_theme)

        group_layout.addWidget(self.lbl_current_theme)
        group_layout.addWidget(btn_toggle_theme)
        group.setLayout(group_layout)

        layout.addWidget(group)
        layout.addStretch()
        widget.setLayout(layout)
        return widget

    def toggle_theme(self):
        ThemeService.toggle()
        self.lbl_current_theme.setText(
            f"Tema Atual: {ThemeService.get_current_theme().upper()}"
        )

    def create_power_tab(self) -> QWidget:
        widget = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(15)

        # Battery Conservation Card
        bat_group = QGroupBox("Modo de Conservação da Bateria (~80%)")
        bat_layout = QVBoxLayout()
        btn_toggle_bat = QPushButton("🔋 Alternar Limite de Carga (80% vs 100%)")
        btn_toggle_bat.clicked.connect(lambda: BatteryService.toggle())
        bat_layout.addWidget(btn_toggle_bat)
        bat_group.setLayout(bat_layout)

        # Power Profile Card
        profile_group = QGroupBox("Perfil de Desempenho / Energia")
        prof_layout = QHBoxLayout()
        btn_saver = QPushButton(" Economia")
        btn_saver.clicked.connect(lambda: PowerProfileService.toggle("-s"))
        btn_bal = QPushButton(" Equilibrado")
        btn_bal.clicked.connect(lambda: PowerProfileService.toggle("-b"))
        btn_perf = QPushButton(" Desempenho")
        btn_perf.clicked.connect(lambda: PowerProfileService.toggle("-p"))

        prof_layout.addWidget(btn_saver)
        prof_layout.addWidget(btn_bal)
        prof_layout.addWidget(btn_perf)
        profile_group.setLayout(prof_layout)

        layout.addWidget(bat_group)
        layout.addWidget(profile_group)
        layout.addStretch()
        widget.setLayout(layout)
        return widget

    def create_idle_tab(self) -> QWidget:
        widget = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(15)

        group = QGroupBox("Inibidor de Suspensão Automática (Swayidle)")
        group_layout = QVBoxLayout()

        btn_toggle_idle = QPushButton("☕ Alternar Inibidor de Suspensão (On / Off)")
        btn_toggle_idle.clicked.connect(lambda: IdleService.toggle())

        group_layout.addWidget(btn_toggle_idle)
        group.setLayout(group_layout)

        layout.addWidget(group)
        layout.addStretch()
        widget.setLayout(layout)
        return widget

    def apply_styles(self):
        self.setStyleSheet(
            f"""
            {styles.QWidget()}
            {styles.QListWidget()}
            {styles.QScrollbar()}
            QTabWidget::pane {{
                border: 1px solid #414868;
                border-radius: 8px;
                background-color: rgba(26, 27, 38, 220);
            }}
            QTabBar::tab {{
                background-color: #24283b;
                color: #a9b1d6;
                padding: 10px 18px;
                margin-right: 4px;
                border-top-left-radius: 6px;
                border-top-right-radius: 6px;
                font-weight: bold;
                font-size: 14px;
            }}
            QTabBar::tab:selected {{
                background-color: #7aa2f7;
                color: #1a1b26;
            }}
            QGroupBox {{
                border: 1px solid #414868;
                border-radius: 8px;
                margin-top: 15px;
                font-weight: bold;
                color: #7aa2f7;
                padding: 15px;
                background: transparent;
            }}
            QPushButton {{
                background-color: #24283b;
                border: 1px solid #414868;
                border-radius: 6px;
                padding: 10px 18px;
                color: #c0caf5;
                font-weight: bold;
                font-size: 13px;
            }}
            QPushButton:hover {{
                background-color: rgba(122, 162, 247, 0.2);
                border-color: #7aa2f7;
            }}
        """
        )
