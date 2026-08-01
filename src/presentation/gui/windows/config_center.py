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
    QLineEdit,
    QCheckBox,
    QFormLayout,
    QScrollArea,
)
from PySide6.QtGui import QIcon, QPixmap, QImage
import presentation.gui.styles as styles
from presentation.gui.components.image_list import CarregadorDeImagens, ListaImagens

# Domain & Application imports
from infrastructure.power.sysfs_battery_repository import SysfsBatteryRepository
from infrastructure.power.swayidle_repository import SwayIdleRepository
from infrastructure.power.powerprofiles_repository import PowerProfilesRepository
from infrastructure.theme.gtk_qt_theme_repository import GtkQtThemeRepository
from infrastructure.theme.sway_wallpaper_repository import SwayWallpaperRepository
from infrastructure.theme.lightdm_repository import LightDMRepository

from application.power.toggle_battery_conservation_use_case import ToggleBatteryConservationUseCase
from application.power.toggle_idle_use_case import ToggleIdleUseCase
from application.power.toggle_power_profile_use_case import TogglePowerProfileUseCase
from application.theme.toggle_theme_use_case import ToggleThemeUseCase
from application.theme.set_wallpaper_use_case import SetWallpaperUseCase
from application.theme.update_lightdm_use_case import UpdateLightDMUseCase
from domain.theme.entities import LightDMSettings


class ConfigCenterWindow(QWidget):
    def __init__(self):
        super().__init__()
        QApplication.setDesktopFileName("sway-manager")
        self.setWindowTitle("SwayManager Control Center")
        self.resize(920, 640)

        # Instantiate Use Cases
        self.battery_use_case = ToggleBatteryConservationUseCase(SysfsBatteryRepository())
        self.idle_use_case = ToggleIdleUseCase(SwayIdleRepository())
        self.power_use_case = TogglePowerProfileUseCase(PowerProfilesRepository())
        self.theme_use_case = ToggleThemeUseCase(GtkQtThemeRepository())
        self.wallpaper_use_case = SetWallpaperUseCase(SwayWallpaperRepository())
        self.lightdm_use_case = UpdateLightDMUseCase(LightDMRepository())

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
        self.tabs.addTab(self.create_lightdm_tab(), "🚦 Login & LightDM")

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

        try:
            self.wallpaper_use_case.execute(caminho_imagem)
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

        group = QGroupBox("Alternar Tema GTK, Qt & Terminal")
        group_layout = QVBoxLayout()

        self.lbl_current_theme = QLabel(
            f"Tema Atual: {self.theme_use_case.get_state().current_theme.upper()}"
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
        self.theme_use_case.execute()
        self.lbl_current_theme.setText(
            f"Tema Atual: {self.theme_use_case.get_state().current_theme.upper()}"
        )

    def create_power_tab(self) -> QWidget:
        widget = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(15)

        bat_group = QGroupBox("Modo de Conservação da Bateria (~80%)")
        bat_layout = QVBoxLayout()
        btn_toggle_bat = QPushButton("🔋 Alternar Limite de Carga (80% vs 100%)")
        btn_toggle_bat.clicked.connect(lambda: self.battery_use_case.execute())
        bat_layout.addWidget(btn_toggle_bat)
        bat_group.setLayout(bat_layout)

        profile_group = QGroupBox("Perfil de Desempenho / Energia")
        prof_layout = QHBoxLayout()
        btn_saver = QPushButton(" Economia")
        btn_saver.clicked.connect(lambda: self.power_use_case.execute("-s"))
        btn_bal = QPushButton(" Equilibrado")
        btn_bal.clicked.connect(lambda: self.power_use_case.execute("-b"))
        btn_perf = QPushButton(" Desempenho")
        btn_perf.clicked.connect(lambda: self.power_use_case.execute("-p"))

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
        btn_toggle_idle.clicked.connect(lambda: self.idle_use_case.execute())

        group_layout.addWidget(btn_toggle_idle)
        group.setLayout(group_layout)

        layout.addWidget(group)
        layout.addStretch()
        widget.setLayout(layout)
        return widget

    def create_lightdm_tab(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("background: transparent; border: none;")

        container = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(15, 15, 15, 15)
        layout.setSpacing(15)

        self.lightdm_settings = self.lightdm_use_case.get_settings().to_dict()
        self.selected_lightdm_bg = None

        bg_group = QGroupBox("🖼️ Plano de Fundo do LightDM (Login)")
        bg_layout = QVBoxLayout()

        self.lbl_lightdm_bg = QLabel(
            f"Imagem Atual: {self.lightdm_settings.get('background', 'Nenhum')}"
        )
        self.lbl_lightdm_bg.setStyleSheet("font-size: 13px; color: #a9b1d6;")

        bg_btn_layout = QHBoxLayout()
        btn_pick_bg = QPushButton("📁 Escolher Imagem")
        btn_pick_bg.clicked.connect(self.selecionar_imagem_lightdm)

        btn_use_current_sway = QPushButton("✨ Usar Wallpaper Atual do Sway")
        btn_use_current_sway.clicked.connect(self.usar_wallpaper_sway_no_lightdm)

        bg_btn_layout.addWidget(btn_pick_bg)
        bg_btn_layout.addWidget(btn_use_current_sway)
        bg_btn_layout.addStretch()

        bg_layout.addWidget(self.lbl_lightdm_bg)
        bg_layout.addLayout(bg_btn_layout)
        bg_group.setLayout(bg_layout)
        layout.addWidget(bg_group)

        style_group = QGroupBox("🎨 Estilo e Aparência do Greeter")
        form_layout = QFormLayout()
        form_layout.setSpacing(10)

        self.txt_gtk_theme = QLineEdit(self.lightdm_settings.get("theme-name", "Adwaita-dark"))
        self.txt_icon_theme = QLineEdit(self.lightdm_settings.get("icon-theme-name", "Adwaita"))
        self.txt_cursor_theme = QLineEdit(self.lightdm_settings.get("cursor-theme-name", "Bibata-Modern-Ice"))
        self.txt_font_name = QLineEdit(self.lightdm_settings.get("font-name", "Sans 11"))
        self.txt_clock_format = QLineEdit(self.lightdm_settings.get("clock-format", "%a, %d %b %H:%M"))

        form_layout.addRow(QLabel("Tema GTK:"), self.txt_gtk_theme)
        form_layout.addRow(QLabel("Tema de Ícones:"), self.txt_icon_theme)
        form_layout.addRow(QLabel("Tema do Cursor:"), self.txt_cursor_theme)
        form_layout.addRow(QLabel("Fonte:"), self.txt_font_name)
        form_layout.addRow(QLabel("Formato do Relógio:"), self.txt_clock_format)

        style_group.setLayout(form_layout)
        layout.addWidget(style_group)

        opt_group = QGroupBox("⚙️ Opções Adicionais")
        opt_layout = QVBoxLayout()

        self.chk_user_bg = QCheckBox("Carregar wallpaper individual do usuário (draw-user-backgrounds)")
        self.chk_user_bg.setChecked(self.lightdm_settings.get("draw-user-backgrounds", "false").lower() == "true")

        self.chk_hide_user_img = QCheckBox("Ocultar foto do usuário na tela de login (hide-user-image)")
        self.chk_hide_user_img.setChecked(self.lightdm_settings.get("hide-user-image", "false").lower() == "true")

        opt_layout.addWidget(self.chk_user_bg)
        opt_layout.addWidget(self.chk_hide_user_img)
        opt_group.setLayout(opt_layout)
        layout.addWidget(opt_group)

        action_layout = QHBoxLayout()
        self.lbl_lightdm_status = QLabel("")
        self.lbl_lightdm_status.setStyleSheet("font-size: 13px; font-weight: bold; color: #9ecc65;")

        btn_save_lightdm = QPushButton("💾 Salvar Configurações no LightDM")
        btn_save_lightdm.clicked.connect(self.salvar_config_lightdm)

        action_layout.addWidget(self.lbl_lightdm_status)
        action_layout.addStretch()
        action_layout.addWidget(btn_save_lightdm)
        layout.addLayout(action_layout)

        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)
        return scroll

    def selecionar_imagem_lightdm(self):
        caminho, _ = QFileDialog.getOpenFileName(
            self,
            "Selecionar Imagem de Fundo para o LightDM",
            self.pasta_imagens,
            "Imagens (*.jpg *.jpeg *.png *.webp)",
        )
        if caminho:
            self.selected_lightdm_bg = caminho
            self.lbl_lightdm_bg.setText(f"Selecionada: {caminho}")

    def usar_wallpaper_sway_no_lightdm(self):
        target = self.wallpaper_use_case.get_current()
        if target:
            self.selected_lightdm_bg = target
            self.lbl_lightdm_bg.setText(f"Selecionada (Sway): {target}")
            self.lbl_lightdm_status.setText(f"✅ Wallpaper do Sway selecionado: {os.path.basename(target)}")
        else:
            self.lbl_lightdm_status.setText("⚠️ Nenhum wallpaper atual do Sway foi encontrado.")

    def salvar_config_lightdm(self):
        dict_settings = {
            "theme-name": self.txt_gtk_theme.text().strip(),
            "icon-theme-name": self.txt_icon_theme.text().strip(),
            "cursor-theme-name": self.txt_cursor_theme.text().strip(),
            "font-name": self.txt_font_name.text().strip(),
            "clock-format": self.txt_clock_format.text().strip(),
            "draw-user-backgrounds": "true" if self.chk_user_bg.isChecked() else "false",
            "hide-user-image": "true" if self.chk_hide_user_img.isChecked() else "false",
        }
        if not self.selected_lightdm_bg:
            dict_settings["background"] = self.lightdm_settings.get("background", "/etc/lightdm/background.jpg")

        new_settings = LightDMSettings.from_dict(dict_settings)
        success = self.lightdm_use_case.execute(new_settings, self.selected_lightdm_bg)
        if success:
            self.lbl_lightdm_status.setText("✅ Configurações salvas no LightDM com sucesso!")
            self.lightdm_settings = self.lightdm_use_case.get_settings().to_dict()
            self.lbl_lightdm_bg.setText(f"Imagem Atual: {self.lightdm_settings.get('background', 'Nenhum')}")
            self.selected_lightdm_bg = None
        else:
            self.lbl_lightdm_status.setText("❌ Erro ao salvar (operação cancelada ou falha).")

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
            QLineEdit {{
                background-color: #24283b;
                border: 1px solid #414868;
                border-radius: 6px;
                padding: 8px 12px;
                color: #c0caf5;
                font-size: 13px;
            }}
            QLineEdit:focus {{
                border-color: #7aa2f7;
            }}
            QCheckBox {{
                color: #c0caf5;
                font-size: 13px;
                spacing: 8px;
            }}
            QLabel {{
                color: #a9b1d6;
                font-size: 13px;
            }}
        """
        )
