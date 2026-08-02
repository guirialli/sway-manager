import os
import subprocess
import datetime
from PySide6.QtCore import Qt, QSize
from PySide6.QtWidgets import (
    QApplication,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QListWidget,
    QListWidgetItem,
    QLineEdit,
    QCheckBox,
    QComboBox,
    QListView,
    QFormLayout,
    QScrollArea,
    QStackedWidget,
    QFrame,
    QFileDialog,
    QSpinBox,
)
from PySide6.QtGui import QPixmap, QImage, QIcon
import presentation.gui.styles as styles
from presentation.gui.components.image_list import CarregadorDeImagens, ListaImagens
from presentation.gui.components.styled_combo_box import ThemeComboBox
from utils.string import StringUtils

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
from application.theme.manage_appearance_use_case import ManageAppearanceUseCase
from application.notification.send_notification_use_case import SendNotificationUseCase
from infrastructure.notification.desktop_notification_repository import DesktopNotificationRepository
from infrastructure.media.grim_slurp_screenshot_repository import GrimSlurpScreenshotRepository
from infrastructure.display.sway_display_repository import SwayDisplayRepository
from application.display.switch_display_mode_use_case import SwitchDisplayModeUseCase
from domain.display.entities import DisplaySwitchType
from domain.theme.entities import LightDMSettings, AppearanceSettings


class ConfigCenterWindow(QWidget):
    def __init__(self):
        super().__init__()
        QApplication.setDesktopFileName("sway-manager")
        self.setWindowTitle("SwayManager Control Center")
        self.resize(920, 620)
        self.setObjectName("mainWindow")

        # Instantiate Use Cases
        self.display_use_case = SwitchDisplayModeUseCase(SwayDisplayRepository())
        self.battery_use_case = ToggleBatteryConservationUseCase(SysfsBatteryRepository())
        self.idle_use_case = ToggleIdleUseCase(SwayIdleRepository())
        self.power_use_case = TogglePowerProfileUseCase(PowerProfilesRepository())
        self.theme_repo = GtkQtThemeRepository()
        self.theme_use_case = ToggleThemeUseCase(self.theme_repo)
        self.appearance_use_case = ManageAppearanceUseCase(self.theme_repo)
        self.wallpaper_use_case = SetWallpaperUseCase(SwayWallpaperRepository())
        self.lightdm_use_case = UpdateLightDMUseCase(LightDMRepository())
        self.notification_use_case = SendNotificationUseCase(DesktopNotificationRepository())
        self.screenshot_repo = GrimSlurpScreenshotRepository()

        self.pasta_imagens = self.wallpaper_use_case.get_wallpaper_folder()
        self.pasta_screenshots = self.screenshot_repo.get_screenshot_folder()

        self.carregador = None
        self.setup_ui()
        self.apply_theme_styles()

    def setup_ui(self):
        root_layout = QHBoxLayout()
        root_layout.setContentsMargins(0, 0, 0, 0)
        root_layout.setSpacing(0)

        # ---------------------------------------------------------
        # macOS Sidebar Panel (Left Side)
        # ---------------------------------------------------------
        sidebar_panel = QFrame()
        sidebar_panel.setFixedWidth(230)
        sidebar_panel.setObjectName("sidebarPanel")

        sidebar_layout = QVBoxLayout()
        sidebar_layout.setContentsMargins(12, 16, 12, 16)
        sidebar_layout.setSpacing(12)

        # App Header Title in Sidebar
        header_title = QLabel("SwayManager")
        header_title.setStyleSheet("font-size: 16px; font-weight: 700; background: transparent; padding-left: 8px;")
        sidebar_layout.addWidget(header_title)

        # Sidebar Menu List
        self.sidebar_list = QListWidget()
        self.sidebar_list.setObjectName("sidebar")
        
        items = [
            ("🖥️  Monitores", 0),
            ("🖼️  Papel de Parede", 1),
            ("🌓  Aparência & Tema", 2),
            ("🔋  Bateria & Energia", 3),
            ("☕  Suspensão & Idle", 4),
            ("🚦  Login & LightDM", 5),
        ]

        for text, index in items:
            item = QListWidgetItem(text)
            item.setData(Qt.UserRole, index)
            self.sidebar_list.addItem(item)

        self.sidebar_list.currentRowChanged.connect(self.switch_page)
        sidebar_layout.addWidget(self.sidebar_list)
        sidebar_panel.setLayout(sidebar_layout)

        root_layout.addWidget(sidebar_panel)

        # ---------------------------------------------------------
        # Content Pages Area (Right Side)
        # ---------------------------------------------------------
        self.stacked_widget = QStackedWidget()
        self.stacked_widget.setObjectName("mainContainer")

        self.stacked_widget.addWidget(self.create_monitors_page())
        self.stacked_widget.addWidget(self.create_wallpaper_page())
        self.stacked_widget.addWidget(self.create_theme_page())
        self.stacked_widget.addWidget(self.create_power_page())
        self.stacked_widget.addWidget(self.create_idle_page())
        self.stacked_widget.addWidget(self.create_lightdm_page())

        root_layout.addWidget(self.stacked_widget)
        self.setLayout(root_layout)

        # Select first tab by default
        self.sidebar_list.setCurrentRow(0)

    def switch_page(self, index: int):
        if 0 <= index < self.stacked_widget.count():
            self.stacked_widget.setCurrentIndex(index)

    def create_card(self, title: str = None, subtitle: str = None) -> tuple[QFrame, QVBoxLayout]:
        card = QFrame()
        card.setObjectName("appleCard")
        card_layout = QVBoxLayout()
        card_layout.setContentsMargins(16, 14, 16, 14)
        card_layout.setSpacing(10)

        if title:
            lbl_title = QLabel(title)
            lbl_title.setStyleSheet("font-size: 13px; font-weight: 600; background: transparent;")
            card_layout.addWidget(lbl_title)
        if subtitle:
            lbl_sub = QLabel(subtitle)
            lbl_sub.setStyleSheet("font-size: 12px; color: #8E8E93; background: transparent;")
            card_layout.addWidget(lbl_sub)

        card.setLayout(card_layout)
        return card, card_layout

    # ---------------------------------------------------------
    # Page 0: Monitores & Telas
    # ---------------------------------------------------------
    def create_monitors_page(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("background: transparent; border: none;")

        container = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(24, 20, 24, 20)
        layout.setSpacing(16)

        header = QLabel("Monitores & Telas")
        header.setStyleSheet("font-size: 20px; font-weight: 700; background: transparent;")
        layout.addWidget(header)

        # Card 1 (FIRST CONFIGURATION AT TOP): Screenshots Folder Configuration
        card_ss, card_ss_layout = self.create_card(
            "📸 Capturas de Tela (Screenshots)",
            "Configuração do diretório onde as capturas de tela serão salvas."
        )

        top_bar = QHBoxLayout()
        self.lbl_pasta_screenshot = QLabel(f"Pasta: {self.pasta_screenshots}")
        self.lbl_pasta_screenshot.setStyleSheet("font-size: 12px; color: #8E8E93; background: transparent;")

        btn_select_ss = QPushButton("📁 Escolher Pasta")
        btn_select_ss.clicked.connect(self.selecionar_pasta_screenshot)

        top_bar.addWidget(self.lbl_pasta_screenshot)
        top_bar.addStretch()
        top_bar.addWidget(btn_select_ss)
        card_ss_layout.addLayout(top_bar)
        layout.addWidget(card_ss)

        # Card 2: Inline Monitor Layout Selection (Only shown if 2 or more monitors connected)
        try:
            mon_count = self.display_use_case.get_connected_monitors_count()
        except Exception:
            mon_count = 1

        if mon_count >= 2:
            card_mon, card_mon_layout = self.create_card(
                "🖥️ Layout de Monitores",
                "Selecione o modo de exibição para seus múltiplos monitores."
            )

            btn_layout = QHBoxLayout()
            btn_layout.setSpacing(12)

            self.layout_buttons = {}
            current_mode = self.display_use_case.get_current_layout()

            options = [
                (DisplaySwitchType.PC_ONLY, "💻\n\nApenas Notebook"),
                (DisplaySwitchType.MONITOR_ONLY, "🖥️\n\nApenas Monitor"),
                (DisplaySwitchType.EXTEND, "🖥️ 🖥️\n\nEstender Telas"),
                (DisplaySwitchType.DUPLICATE, "🖥️ = 🖥️\n\nDuplicar Telas"),
            ]

            for mode, label_text in options:
                btn = QPushButton(label_text)
                btn.setMinimumHeight(70)
                btn.setCursor(Qt.PointingHandCursor)
                btn.clicked.connect(lambda _, m=mode: self.aplicar_layout_monitor(m))
                self.layout_buttons[mode] = btn
                btn_layout.addWidget(btn)

            card_mon_layout.addLayout(btn_layout)
            layout.addWidget(card_mon)
            self.atualizar_destaque_layout_monitores(current_mode)

        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)
        return scroll

    def aplicar_layout_monitor(self, mode: DisplaySwitchType):
        try:
            self.display_use_case.execute(mode)
            self.atualizar_destaque_layout_monitores(mode)
        except Exception as e:
            print(f"Erro ao aplicar layout de monitor: {e}")

    def atualizar_destaque_layout_monitores(self, active_mode: DisplaySwitchType):
        if not hasattr(self, "layout_buttons"):
            return

        c = styles.get_colors(self.theme_repo.get_state().current_theme)
        for mode, btn in self.layout_buttons.items():
            if mode == active_mode:
                btn.setStyleSheet(f"""
                    QPushButton {{
                        background-color: {c['accent']};
                        border: 2px solid {c['accent']};
                        border-radius: 10px;
                        padding: 12px;
                        font-size: 13px;
                        font-weight: 700;
                        color: {c['accent_text']};
                    }}
                """)
            else:
                btn.setStyleSheet(f"""
                    QPushButton {{
                        background-color: {c['button_bg']};
                        border: 1px solid {c['button_border']};
                        border-radius: 10px;
                        padding: 12px;
                        font-size: 13px;
                        font-weight: 500;
                        color: {c['text_primary']};
                    }}
                    QPushButton:hover {{
                        background-color: {c['accent']};
                        border-color: {c['accent']};
                        color: {c['accent_text']};
                    }}
                """)

    def selecionar_pasta_screenshot(self):
        nova_pasta = QFileDialog.getExistingDirectory(
            self, "Selecionar Pasta de Capturas de Tela", self.pasta_screenshots
        )
        if nova_pasta:
            self.pasta_screenshots = nova_pasta
            self.screenshot_repo.set_screenshot_folder(nova_pasta)
            self.lbl_pasta_screenshot.setText(f"Pasta: {self.pasta_screenshots}")

    # ---------------------------------------------------------
    # Page 1: Papel de Parede
    # ---------------------------------------------------------
    def create_wallpaper_page(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("background: transparent; border: none;")

        container = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(24, 20, 24, 20)
        layout.setSpacing(16)

        header = QLabel("Papel de Parede")
        header.setStyleSheet("font-size: 20px; font-weight: 700; background: transparent;")
        layout.addWidget(header)

        # Inset Card: Folder Selection & Grid
        card, card_layout = self.create_card("Biblioteca de Imagens")

        top_bar = QHBoxLayout()
        self.lbl_pasta = QLabel(f"Pasta: {self.pasta_imagens}")
        self.lbl_pasta.setStyleSheet("font-size: 12px; color: #8E8E93; background: transparent;")

        btn_select = QPushButton("📁 Escolher Pasta")
        btn_select.clicked.connect(self.selecionar_pasta)

        top_bar.addWidget(self.lbl_pasta)
        top_bar.addStretch()
        top_bar.addWidget(btn_select)
        card_layout.addLayout(top_bar)

        self.lista_imagens = ListaImagens(self.aplicar_wallpaper, None)
        self.lista_imagens.setViewMode(QListWidget.ViewMode.IconMode)
        self.lista_imagens.setIconSize(QSize(216, 122))
        self.lista_imagens.setResizeMode(QListWidget.ResizeMode.Adjust)
        self.lista_imagens.setMovement(QListWidget.Movement.Static)
        self.lista_imagens.setSpacing(12)
        self.lista_imagens.setMinimumHeight(380)
        self.lista_imagens.itemDoubleClicked.connect(self.aplicar_wallpaper)
        card_layout.addWidget(self.lista_imagens)

        # Action bar inside card
        bottom_bar = QHBoxLayout()
        btn_aplicar = QPushButton("✨ Aplicar Papel de Parede")
        btn_aplicar.setProperty("class", "primaryButton")
        btn_aplicar.clicked.connect(self.aplicar_wallpaper_selecionado)
        bottom_bar.addStretch()
        bottom_bar.addWidget(btn_aplicar)
        card_layout.addLayout(bottom_bar)

        layout.addWidget(card)
        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)

        self.iniciar_carregador_imagens()
        return scroll

    def selecionar_pasta(self):
        nova_pasta = QFileDialog.getExistingDirectory(
            self, "Selecionar Pasta de Wallpapers", self.pasta_imagens
        )
        if nova_pasta:
            self.pasta_imagens = nova_pasta
            self.wallpaper_use_case.set_wallpaper_folder(nova_pasta)
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
        item.setSizeHint(QSize(228, 150))
        
        nome_completo = os.path.basename(caminho)
        item.setToolTip(nome_completo)
        item.setText(StringUtils.truncar_nome_arquivo(nome_completo, max_len=20))
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
            self.notification_use_case.execute(
                title="Papel de Parede",
                message=f"Aplicado: {os.path.basename(caminho_imagem)}",
            )
        except Exception as e:
            print(f"Erro ao aplicar wallpaper: {e}")

    def aplicar_wallpaper_selecionado(self):
        item = self.lista_imagens.currentItem()
        if item:
            self.aplicar_wallpaper(item)

    # ---------------------------------------------------------
    # Page 2: Aparência & Tema
    # ---------------------------------------------------------
    def create_theme_page(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("background: transparent; border: none;")

        container = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(24, 20, 24, 20)
        layout.setSpacing(16)

        header = QLabel("Aparência & Tema")
        header.setStyleSheet("font-size: 20px; font-weight: 700; background: transparent;")
        layout.addWidget(header)

        # Card: System Theme Toggle
        card, card_layout = self.create_card(
            "Tema do Sistema (GTK, Qt & Terminal)",
            "Alterne instantaneamente a aparência entre os modos Escuro e Claro do sistema."
        )

        theme_row = QHBoxLayout()
        self.lbl_current_theme = QLabel(
            f"Tema Atual: {self.theme_use_case.get_state().current_theme.upper()}"
        )
        self.lbl_current_theme.setStyleSheet(
            "font-size: 14px; font-weight: 600; background: transparent;"
        )

        btn_toggle_theme = QPushButton("🌓 Alternar Tema (Dark / Light)")
        btn_toggle_theme.setProperty("class", "primaryButton")
        btn_toggle_theme.clicked.connect(self.toggle_theme)

        theme_row.addWidget(self.lbl_current_theme)
        theme_row.addStretch()
        theme_row.addWidget(btn_toggle_theme)

        card_layout.addLayout(theme_row)
        layout.addWidget(card)

        # Card 2: GTK, Icons, Cursors & Fonts Customization
        card_app, app_layout = self.create_card(
            "Personalização de Aparência (GTK, Ícones, Cursor e Fonte)",
            "Selecione o tema GTK, tema de ícones, tema do cursor e fonte do sistema entre as opções instaladas."
        )

        form_layout = QFormLayout()
        form_layout.setSpacing(12)

        options = self.appearance_use_case.get_available_options()
        current_settings = self.appearance_use_case.get_settings()

        # ComboBox Tema GTK
        self.combo_gtk_theme = ThemeComboBox(options.gtk_themes, current_settings.gtk_theme)

        # ComboBox Tema de Ícones
        self.combo_icon_theme = ThemeComboBox(options.icon_themes, current_settings.icon_theme)

        # ComboBox Tema do Cursor
        self.combo_cursor_theme = ThemeComboBox(options.cursor_themes, current_settings.cursor_theme)

        # ComboBox Fonte do Sistema
        self.combo_font = ThemeComboBox(options.fonts, current_settings.font_name)

        form_layout.addRow(QLabel("Tema GTK:"), self.combo_gtk_theme)
        form_layout.addRow(QLabel("Tema de Ícones:"), self.combo_icon_theme)
        form_layout.addRow(QLabel("Tema do Cursor:"), self.combo_cursor_theme)
        form_layout.addRow(QLabel("Fonte do Sistema:"), self.combo_font)

        app_layout.addLayout(form_layout)

        # Action row for applying appearance settings
        app_action_row = QHBoxLayout()
        self.lbl_appearance_status = QLabel("")
        self.lbl_appearance_status.setStyleSheet("font-size: 13px; font-weight: 600; color: #34C759;")

        btn_apply_appearance = QPushButton("✨ Aplicar Aparência")
        btn_apply_appearance.setProperty("class", "primaryButton")
        btn_apply_appearance.clicked.connect(self.aplicar_config_aparencia)

        app_action_row.addWidget(self.lbl_appearance_status)
        app_action_row.addStretch()
        app_action_row.addWidget(btn_apply_appearance)

        app_layout.addLayout(app_action_row)
        layout.addWidget(card_app)

        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)
        return scroll

    def toggle_theme(self):
        self.theme_use_case.execute()
        current_mode = self.theme_use_case.get_state().current_theme
        self.lbl_current_theme.setText(f"Tema Atual: {current_mode.upper()}")
        self.apply_theme_styles()

    def aplicar_config_aparencia(self):
        new_settings = AppearanceSettings(
            gtk_theme=self.combo_gtk_theme.currentText(),
            icon_theme=self.combo_icon_theme.currentText(),
            cursor_theme=self.combo_cursor_theme.currentText(),
            font_name=self.combo_font.currentText(),
        )
        success = self.appearance_use_case.apply(new_settings)
        if success:
            self.lbl_appearance_status.setText("✅ Aparência atualizada com sucesso!")
        else:
            self.lbl_appearance_status.setText("❌ Erro ao aplicar configurações de aparência.")


    # ---------------------------------------------------------
    # Page 3: Bateria & Energia
    # ---------------------------------------------------------
    def create_power_page(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("background: transparent; border: none;")

        container = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(24, 20, 24, 20)
        layout.setSpacing(16)

        header = QLabel("Bateria & Energia")
        header.setStyleSheet("font-size: 20px; font-weight: 700; background: transparent;")
        layout.addWidget(header)

        # Card 1: Battery Limit Conservation Mode
        card_bat, bat_layout = self.create_card(
            "Modo de Conservação da Bateria",
            "Limita a carga máxima em ~80% para preservar a vida útil da bateria."
        )

        bat_row = QHBoxLayout()
        btn_toggle_bat = QPushButton("🔋 Alternar Limite de Carga (80% vs 100%)")
        btn_toggle_bat.clicked.connect(lambda: self.battery_use_case.execute())
        bat_row.addStretch()
        bat_row.addWidget(btn_toggle_bat)
        bat_layout.addLayout(bat_row)
        layout.addWidget(card_bat)

        # Card 2: Power Profiles
        card_prof, prof_layout = self.create_card(
            "Perfil de Desempenho",
            "Ajuste o comportamento do processador para priorizar economia de energia ou velocidade."
        )

        btn_row = QHBoxLayout()
        btn_saver = QPushButton("🌱 Economia")
        btn_saver.clicked.connect(lambda: self.power_use_case.execute("-s"))
        btn_bal = QPushButton("⚖️ Equilibrado")
        btn_bal.clicked.connect(lambda: self.power_use_case.execute("-b"))
        btn_perf = QPushButton("⚡ Desempenho")
        btn_perf.clicked.connect(lambda: self.power_use_case.execute("-p"))

        btn_row.addWidget(btn_saver)
        btn_row.addWidget(btn_bal)
        btn_row.addWidget(btn_perf)
        prof_layout.addLayout(btn_row)

        layout.addWidget(card_prof)

        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)
        return scroll

    # ---------------------------------------------------------
    # Page 4: Suspensão & Idle
    # ---------------------------------------------------------
    def create_idle_page(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("background: transparent; border: none;")

        container = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(24, 20, 24, 20)
        layout.setSpacing(16)

        header = QLabel("Suspensão & Idle")
        header.setStyleSheet("font-size: 20px; font-weight: 700; background: transparent;")
        layout.addWidget(header)

        card, card_layout = self.create_card(
            "Inibidor de Suspensão Automática (Swayidle)",
            "Impede que o sistema entre em suspensão ou desligue a tela automaticamente."
        )

        idle_row = QHBoxLayout()
        btn_toggle_idle = QPushButton("☕ Alternar Inibidor (Ativo / Inativo)")
        btn_toggle_idle.clicked.connect(lambda: self.idle_use_case.execute())
        idle_row.addStretch()
        idle_row.addWidget(btn_toggle_idle)

        card_layout.addLayout(idle_row)
        layout.addWidget(card)

        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)
        return scroll

    # ---------------------------------------------------------
    # Page 5: Login & LightDM
    # ---------------------------------------------------------
    def create_lightdm_page(self) -> QWidget:
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("background: transparent; border: none;")

        container = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(24, 20, 24, 20)
        layout.setSpacing(16)

        header = QLabel("Login & LightDM")
        header.setStyleSheet("font-size: 20px; font-weight: 700; background: transparent;")
        layout.addWidget(header)

        self.lightdm_settings = self.lightdm_use_case.get_settings().to_dict()
        self.selected_lightdm_bg = None

        available_users = self.lightdm_use_case.get_available_users()
        available_sessions = self.lightdm_use_case.get_available_sessions()
        options = self.appearance_use_case.get_available_options()

        # ---------------------------------------------------------
        # Card 0: Live Greeter Preview
        # ---------------------------------------------------------
        card_preview, preview_card_layout = self.create_card("Pré-visualização ao Vivo do Greeter")
        
        self.preview_frame = QFrame()
        self.preview_frame.setFixedHeight(160)
        self.preview_frame.setStyleSheet(
            "QFrame { background-color: #1e1e2e; border-radius: 8px; border: 1px solid #313244; }"
        )
        
        preview_box_layout = QVBoxLayout(self.preview_frame)
        preview_box_layout.setContentsMargins(16, 12, 16, 12)

        # Top bar of preview
        top_bar = QHBoxLayout()
        lbl_host = QLabel("🖥️ sway-desktop")
        lbl_host.setStyleSheet("font-size: 11px; color: #a6adc8; font-weight: 600;")
        self.lbl_preview_clock = QLabel("")
        self.lbl_preview_clock.setStyleSheet("font-size: 12px; color: #cdd6f4; font-weight: 700;")
        
        top_bar.addWidget(lbl_host)
        top_bar.addStretch()
        top_bar.addWidget(self.lbl_preview_clock)
        preview_box_layout.addLayout(top_bar)
        preview_box_layout.addStretch()

        # Center login box of preview
        center_login = QHBoxLayout()
        center_card = QFrame()
        center_card.setFixedWidth(260)
        center_card.setStyleSheet(
            "QFrame { background: rgba(30, 30, 46, 0.85); border-radius: 8px; border: 1px solid rgba(255,255,255,0.15); }"
        )
        center_layout = QVBoxLayout(center_card)
        center_layout.setContentsMargins(12, 10, 12, 10)
        center_layout.setSpacing(6)

        avatar_row = QHBoxLayout()
        lbl_avatar = QLabel("👤")
        lbl_avatar.setStyleSheet("font-size: 22px; background: transparent;")
        
        user_name_text = self.lightdm_settings.get("autologin-user") or (available_users[0] if available_users else "Usuario")
        self.lbl_preview_username = QLabel(user_name_text)
        self.lbl_preview_username.setStyleSheet("font-size: 13px; font-weight: 700; color: #ffffff;")
        
        avatar_row.addWidget(lbl_avatar)
        avatar_row.addWidget(self.lbl_preview_username)
        avatar_row.addStretch()
        center_layout.addLayout(avatar_row)

        lbl_pass_sim = QLabel("••••••••••••")
        lbl_pass_sim.setStyleSheet("background: rgba(0,0,0,0.3); color: #888; border-radius: 4px; padding: 4px; font-size: 10px;")
        center_layout.addWidget(lbl_pass_sim)

        self.lbl_preview_session = QLabel(f"Sessão: {self.lightdm_settings.get('user-session', 'sway')}")
        self.lbl_preview_session.setStyleSheet("font-size: 10px; color: #a6adc8;")
        center_layout.addWidget(self.lbl_preview_session)

        center_login.addStretch()
        center_login.addWidget(center_card)
        center_login.addStretch()

        preview_box_layout.addLayout(center_login)
        preview_box_layout.addStretch()

        preview_card_layout.addWidget(self.preview_frame)
        layout.addWidget(card_preview)

        # ---------------------------------------------------------
        # Card 1: Background
        # ---------------------------------------------------------
        card_bg, bg_layout = self.create_card("Plano de Fundo da Tela de Login")

        bg_actual = self.lightdm_settings.get("background", "Nenhum")
        txt, tooltip = self.format_lightdm_bg_text("Imagem Atual", bg_actual)
        self.lbl_lightdm_bg = QLabel(txt)
        if tooltip:
            self.lbl_lightdm_bg.setToolTip(tooltip)
        self.lbl_lightdm_bg.setStyleSheet("font-size: 12px; color: #8E8E93; background: transparent;")

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
        layout.addWidget(card_bg)

        # ---------------------------------------------------------
        # Card 2: Daemon & Autologin Settings (lightdm.conf)
        # ---------------------------------------------------------
        card_daemon, daemon_layout = self.create_card(
            "Login Automático & Sessão do Sistema (lightdm.conf)",
            "Configurações gerais do servidor LightDM, autologin e permissões de sessão."
        )
        daemon_form = QFormLayout()
        daemon_form.setSpacing(10)

        # Autologin toggle
        self.chk_autologin_enable = QCheckBox("Ativar Login Automático sem pedir Senha")
        curr_autologin_user = self.lightdm_settings.get("autologin-user", "")
        self.chk_autologin_enable.setChecked(bool(curr_autologin_user))

        # Autologin user dropdown
        self.cmb_autologin_user = ThemeComboBox(available_users, curr_autologin_user or (available_users[0] if available_users else ""), editable=False)
        self.cmb_autologin_user.setEnabled(bool(curr_autologin_user))
        self.chk_autologin_enable.toggled.connect(self.cmb_autologin_user.setEnabled)
        self.cmb_autologin_user.currentTextChanged.connect(self.update_lightdm_preview)

        # Autologin timeout spinbox
        self.spn_autologin_timeout = QSpinBox()
        self.spn_autologin_timeout.setRange(0, 300)
        self.spn_autologin_timeout.setSuffix(" seg")
        try:
            self.spn_autologin_timeout.setValue(int(self.lightdm_settings.get("autologin-user-timeout", "0")))
        except ValueError:
            self.spn_autologin_timeout.setValue(0)

        # Autologin session dropdown
        curr_autologin_sess = self.lightdm_settings.get("autologin-session", "sway")
        self.cmb_autologin_session = ThemeComboBox(available_sessions, curr_autologin_sess, editable=False)

        # User session dropdown
        curr_user_sess = self.lightdm_settings.get("user-session", "sway")
        self.cmb_user_session = ThemeComboBox(available_sessions, curr_user_sess, editable=False)
        self.cmb_user_session.currentTextChanged.connect(self.update_lightdm_preview)

        daemon_form.addRow(self.chk_autologin_enable)
        daemon_form.addRow(QLabel("Usuário do Autologin:"), self.cmb_autologin_user)
        daemon_form.addRow(QLabel("Tempo de Espera Autologin:"), self.spn_autologin_timeout)
        daemon_form.addRow(QLabel("Sessão do Autologin:"), self.cmb_autologin_session)
        daemon_form.addRow(QLabel("Sessão Padrão do Usuário:"), self.cmb_user_session)

        # Additional daemon checkboxes
        self.chk_manual_login = QCheckBox("Permitir digitação manual de nome de usuário (greeter-show-manual-login)")
        self.chk_manual_login.setChecked(self.lightdm_settings.get("greeter-show-manual-login", "false").lower() == "true")

        self.chk_hide_user_list = QCheckBox("Ocultar lista de usuários no login (greeter-hide-users)")
        self.chk_hide_user_list.setChecked(self.lightdm_settings.get("greeter-hide-users", "false").lower() == "true")

        self.chk_allow_guest = QCheckBox("Permitir conta de convidado / Guest (allow-guest)")
        self.chk_allow_guest.setChecked(self.lightdm_settings.get("allow-guest", "false").lower() == "true")

        daemon_layout.addLayout(daemon_form)
        daemon_layout.addWidget(self.chk_manual_login)
        daemon_layout.addWidget(self.chk_hide_user_list)
        daemon_layout.addWidget(self.chk_allow_guest)
        layout.addWidget(card_daemon)

        # ---------------------------------------------------------
        # Card 3: Greeter Styles Form (lightdm-gtk-greeter.conf)
        # ---------------------------------------------------------
        card_style, style_layout = self.create_card(
            "Estilo e Aparência do Greeter (lightdm-gtk-greeter.conf)",
            "Configure o tema GTK, tema de ícones, cursor, fonte e renderização Xft."
        )

        form_layout = QFormLayout()
        form_layout.setSpacing(10)

        self.txt_gtk_theme = ThemeComboBox(
            options.gtk_themes,
            self.lightdm_settings.get("theme-name", "Adwaita-dark"),
            editable=False,
        )

        self.txt_icon_theme = ThemeComboBox(
            options.icon_themes,
            self.lightdm_settings.get("icon-theme-name", "Adwaita"),
            editable=False,
        )

        self.txt_cursor_theme = ThemeComboBox(
            options.cursor_themes,
            self.lightdm_settings.get("cursor-theme-name", "Bibata-Modern-Ice"),
            editable=False,
        )

        cursor_sizes = ["16", "24", "32", "48"]
        self.txt_cursor_size = ThemeComboBox(
            cursor_sizes,
            self.lightdm_settings.get("cursor-theme-size", "24"),
            editable=False,
        )

        self.txt_font_name = ThemeComboBox(
            options.fonts,
            self.lightdm_settings.get("font-name", "Sans 11"),
            editable=False,
        )

        # Xft options
        self.chk_xft_antialias = QCheckBox("Ativar Suavização de Fontes (xft-antialias)")
        self.chk_xft_antialias.setChecked(self.lightdm_settings.get("xft-antialias", "true").lower() == "true")

        self.txt_xft_dpi = QLineEdit(self.lightdm_settings.get("xft-dpi", ""))
        self.txt_xft_dpi.setPlaceholderText("Padrão do Sistema (ex: 96)")

        hint_styles = ["hintslight", "hintnone", "hintmedium", "hintfull"]
        self.cmb_xft_hintstyle = ThemeComboBox(
            hint_styles,
            self.lightdm_settings.get("xft-hintstyle", "hintslight"),
            editable=False,
        )

        rgba_styles = ["rgb", "none", "bgr", "vrgb", "vbgr"]
        self.cmb_xft_rgba = ThemeComboBox(
            rgba_styles,
            self.lightdm_settings.get("xft-rgba", "rgb"),
            editable=False,
        )

        form_layout.addRow(QLabel("Tema GTK:"), self.txt_gtk_theme)
        form_layout.addRow(QLabel("Tema de Ícones:"), self.txt_icon_theme)
        form_layout.addRow(QLabel("Tema do Cursor:"), self.txt_cursor_theme)
        form_layout.addRow(QLabel("Tamanho do Cursor:"), self.txt_cursor_size)
        form_layout.addRow(QLabel("Fonte:"), self.txt_font_name)
        form_layout.addRow(self.chk_xft_antialias)
        form_layout.addRow(QLabel("Resolução DPI (Xft):"), self.txt_xft_dpi)
        form_layout.addRow(QLabel("Estilo de Hinting (Xft):"), self.cmb_xft_hintstyle)
        form_layout.addRow(QLabel("Renderização Subpixel (RGBA):"), self.cmb_xft_rgba)

        style_layout.addLayout(form_layout)

        # Sync button row
        sync_row = QHBoxLayout()
        btn_sync_appearance = QPushButton("✨ Sincronizar Aparência com Sway/Sistema")
        btn_sync_appearance.clicked.connect(self.sincronizar_aparencia_lightdm)
        sync_row.addStretch()
        sync_row.addWidget(btn_sync_appearance)
        style_layout.addLayout(sync_row)

        layout.addWidget(card_style)

        # ---------------------------------------------------------
        # Card 4: Clock, Layout & Indicators
        # ---------------------------------------------------------
        card_layout_opt, layout_opt_layout = self.create_card("Relógio, Posição e Indicadores do Painel")

        form_clock = QFormLayout()
        form_clock.setSpacing(10)

        self.txt_clock_format = QLineEdit(self.lightdm_settings.get("clock-format", "%a, %d %b %H:%M"))
        self.txt_clock_format.textChanged.connect(self.update_lightdm_preview)

        positions = ["50%,center", "50%,50%", "30%,50%", "70%,50%", "50%,30%", "50%,70%"]
        self.cmb_position = ThemeComboBox(
            positions,
            self.lightdm_settings.get("position", "50%,center"),
            editable=True,
        )

        self.txt_active_monitor = QLineEdit(self.lightdm_settings.get("active-monitor", "#cursor"))

        self.spn_screensaver_timeout = QSpinBox()
        self.spn_screensaver_timeout.setRange(0, 3600)
        self.spn_screensaver_timeout.setSuffix(" seg")
        try:
            self.spn_screensaver_timeout.setValue(int(self.lightdm_settings.get("screensaver-timeout", "60")))
        except ValueError:
            self.spn_screensaver_timeout.setValue(60)

        self.txt_indicators = QLineEdit(self.lightdm_settings.get("indicators", "~host;~spacer;~clock;~spacer;~session;~language;~a11y;~power"))

        form_clock.addRow(QLabel("Formato do Relógio:"), self.txt_clock_format)
        form_clock.addRow(QLabel("Posição da Janela de Login:"), self.cmb_position)
        form_clock.addRow(QLabel("Monitor Ativo:"), self.txt_active_monitor)
        form_clock.addRow(QLabel("Timeout de Descanso de Tela:"), self.spn_screensaver_timeout)
        form_clock.addRow(QLabel("Indicadores do Painel Superior:"), self.txt_indicators)

        layout_opt_layout.addLayout(form_clock)
        layout.addWidget(card_layout_opt)

        # ---------------------------------------------------------
        # Card 5: Display & Avatar Options
        # ---------------------------------------------------------
        card_opt, opt_layout = self.create_card("Opções de Exibição de Fundo e Avatares")

        self.chk_user_bg = QCheckBox("Carregar wallpaper individual do usuário (draw-user-backgrounds)")
        self.chk_user_bg.setChecked(self.lightdm_settings.get("draw-user-backgrounds", "false").lower() == "true")

        self.chk_draw_grid = QCheckBox("Desenhar grade sobreposta no plano de fundo (draw-grid)")
        self.chk_draw_grid.setChecked(self.lightdm_settings.get("draw-grid", "false").lower() == "true")

        self.chk_hide_user_img = QCheckBox("Ocultar foto do usuário na tela de login (hide-user-image)")
        self.chk_hide_user_img.setChecked(self.lightdm_settings.get("hide-user-image", "false").lower() == "true")

        opt_form = QFormLayout()
        self.txt_default_user_image = QLineEdit(self.lightdm_settings.get("default-user-image", ""))
        self.txt_default_user_image.setPlaceholderText("Caminho para avatar padrão (ex: /usr/share/pixmaps/avatar.png)")
        btn_pick_avatar = QPushButton("📁 Escolher")
        btn_pick_avatar.clicked.connect(self.selecionar_avatar_padrao_lightdm)
        
        avatar_row = QHBoxLayout()
        avatar_row.addWidget(self.txt_default_user_image)
        avatar_row.addWidget(btn_pick_avatar)
        opt_form.addRow(QLabel("Avatar Padrão do Usuário:"), avatar_row)

        opt_layout.addWidget(self.chk_user_bg)
        opt_layout.addWidget(self.chk_draw_grid)
        opt_layout.addWidget(self.chk_hide_user_img)
        opt_layout.addLayout(opt_form)
        layout.addWidget(card_opt)

        # ---------------------------------------------------------
        # Action Bar at bottom
        # ---------------------------------------------------------
        action_layout = QHBoxLayout()
        btn_save_lightdm = QPushButton("💾 Salvar Configurações no LightDM")
        btn_save_lightdm.setProperty("class", "primaryButton")
        btn_save_lightdm.clicked.connect(self.salvar_config_lightdm)

        action_layout.addStretch()
        action_layout.addWidget(btn_save_lightdm)
        layout.addLayout(action_layout)

        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)

        self.update_lightdm_preview()
        return scroll

    def update_lightdm_preview(self):
        if not hasattr(self, "lbl_preview_clock"):
            return
        
        # 1. Update clock
        fmt = self.txt_clock_format.text().strip() or "%a, %d %b %H:%M"
        try:
            now_str = datetime.datetime.now().strftime(fmt)
            self.lbl_preview_clock.setText(now_str)
        except Exception:
            self.lbl_preview_clock.setText(datetime.datetime.now().strftime("%H:%M"))

        # 2. Update Username & Session labels
        if hasattr(self, "cmb_autologin_user") and hasattr(self, "lbl_preview_username"):
            usr = self.cmb_autologin_user.currentText().strip() if self.chk_autologin_enable.isChecked() else "Usuario"
            self.lbl_preview_username.setText(usr or "Usuario")

        if hasattr(self, "cmb_user_session") and hasattr(self, "lbl_preview_session"):
            sess = self.cmb_user_session.currentText().strip() or "sway"
            self.lbl_preview_session.setText(f"Sessão: {sess}")

    def format_lightdm_bg_text(self, prefix: str, path: str) -> tuple[str, str]:
        if not path or path == "Nenhum":
            return f"{prefix}: Nenhum", ""
        nome_arquivo = os.path.basename(path)
        nome_truncado = StringUtils.truncar_nome_arquivo(nome_arquivo, max_len=30)
        return f"{prefix}: {nome_truncado}", path

    def selecionar_imagem_lightdm(self):
        caminho, _ = QFileDialog.getOpenFileName(
            self,
            "Selecionar Imagem de Fundo para o LightDM",
            self.pasta_imagens,
            "Imagens (*.jpg *.jpeg *.png *.webp)",
        )
        if caminho:
            self.selected_lightdm_bg = caminho
            txt, tooltip = self.format_lightdm_bg_text("Selecionada", caminho)
            self.lbl_lightdm_bg.setText(txt)
            self.lbl_lightdm_bg.setToolTip(tooltip)
            self.update_lightdm_preview()

    def selecionar_avatar_padrao_lightdm(self):
        caminho, _ = QFileDialog.getOpenFileName(
            self,
            "Selecionar Avatar Padrão para o LightDM",
            self.pasta_imagens,
            "Imagens (*.png *.jpg *.jpeg *.svg)",
        )
        if caminho:
            self.txt_default_user_image.setText(caminho)

    def usar_wallpaper_sway_no_lightdm(self):
        target = self.wallpaper_use_case.get_current()
        if target:
            self.selected_lightdm_bg = target
            txt, tooltip = self.format_lightdm_bg_text("Selecionada (Sway)", target)
            self.lbl_lightdm_bg.setText(txt)
            self.lbl_lightdm_bg.setToolTip(tooltip)
            nome_trunc = StringUtils.truncar_nome_arquivo(os.path.basename(target), max_len=25)
            self.notification_use_case.execute(
                title="SwayManager LightDM",
                message=f"Wallpaper do Sway selecionado: {nome_trunc}",
            )
            self.update_lightdm_preview()
        else:
            self.notification_use_case.execute(
                title="SwayManager LightDM",
                message="Nenhum wallpaper atual do Sway foi encontrado.",
            )

    def sincronizar_aparencia_lightdm(self):
        curr = self.appearance_use_case.get_settings()
        if curr.gtk_theme:
            self.txt_gtk_theme.setCurrentText(curr.gtk_theme)
        if curr.icon_theme:
            self.txt_icon_theme.setCurrentText(curr.icon_theme)
        if curr.cursor_theme:
            self.txt_cursor_theme.setCurrentText(curr.cursor_theme)
        if curr.font_name:
            self.txt_font_name.setCurrentText(curr.font_name)

        # Sync Sway wallpaper as well
        target_wp = self.wallpaper_use_case.get_current()
        if target_wp:
            self.selected_lightdm_bg = target_wp
            txt, tooltip = self.format_lightdm_bg_text("Selecionada (Sway)", target_wp)
            self.lbl_lightdm_bg.setText(txt)
            self.lbl_lightdm_bg.setToolTip(tooltip)

        self.update_lightdm_preview()
        self.notification_use_case.execute(
            title="SwayManager LightDM",
            message="Tema, ícones, cursor, fonte e wallpaper do Sway sincronizados no formulário!",
        )

    def salvar_config_lightdm(self):
        autologin_user = self.cmb_autologin_user.currentText().strip() if self.chk_autologin_enable.isChecked() else ""

        dict_settings = {
            # Greeter - Aparência
            "theme-name": self.txt_gtk_theme.currentText().strip(),
            "icon-theme-name": self.txt_icon_theme.currentText().strip(),
            "cursor-theme-name": self.txt_cursor_theme.currentText().strip(),
            "cursor-theme-size": self.txt_cursor_size.currentText().strip(),
            "font-name": self.txt_font_name.currentText().strip(),
            "xft-antialias": "true" if self.chk_xft_antialias.isChecked() else "false",
            "xft-dpi": self.txt_xft_dpi.text().strip(),
            "xft-hintstyle": self.cmb_xft_hintstyle.currentText().strip(),
            "xft-rgba": self.cmb_xft_rgba.currentText().strip(),

            # Greeter - Layout, Relógio e Indicadores
            "clock-format": self.txt_clock_format.text().strip(),
            "position": self.cmb_position.currentText().strip(),
            "active-monitor": self.txt_active_monitor.text().strip(),
            "screensaver-timeout": str(self.spn_screensaver_timeout.value()),
            "indicators": self.txt_indicators.text().strip(),

            # Greeter - Exibição e Usuários
            "draw-user-backgrounds": "true" if self.chk_user_bg.isChecked() else "false",
            "draw-grid": "true" if self.chk_draw_grid.isChecked() else "false",
            "hide-user-image": "true" if self.chk_hide_user_img.isChecked() else "false",
            "default-user-image": self.txt_default_user_image.text().strip(),

            # Daemon / Seat - Autologin & Sessão
            "autologin-user": autologin_user,
            "autologin-user-timeout": str(self.spn_autologin_timeout.value()),
            "autologin-session": self.cmb_autologin_session.currentText().strip(),
            "user-session": self.cmb_user_session.currentText().strip(),
            "greeter-show-manual-login": "true" if self.chk_manual_login.isChecked() else "false",
            "greeter-hide-users": "true" if self.chk_hide_user_list.isChecked() else "false",
            "allow-guest": "true" if self.chk_allow_guest.isChecked() else "false",
        }

        if not self.selected_lightdm_bg:
            dict_settings["background"] = self.lightdm_settings.get("background", "/etc/lightdm/background.jpg")

        new_settings = LightDMSettings.from_dict(dict_settings)
        success = self.lightdm_use_case.execute(new_settings, self.selected_lightdm_bg)
        if success:
            self.notification_use_case.execute(
                title="SwayManager LightDM",
                message="Configurações salvas no LightDM com sucesso!",
            )
            self.lightdm_settings = self.lightdm_use_case.get_settings().to_dict()
            bg_path = self.lightdm_settings.get("background", "Nenhum")
            txt, tooltip = self.format_lightdm_bg_text("Imagem Atual", bg_path)
            self.lbl_lightdm_bg.setText(txt)
            self.lbl_lightdm_bg.setToolTip(tooltip)
            self.selected_lightdm_bg = None
            self.update_lightdm_preview()
        else:
            self.notification_use_case.execute(
                title="SwayManager LightDM",
                message="Erro ao salvar (operação cancelada ou falha).",
                urgency="critical",
            )

    def apply_theme_styles(self):
        current_mode = self.theme_use_case.get_state().current_theme
        c = styles.get_colors(current_mode)
        palette = styles.get_palette(current_mode)

        if QApplication.instance():
            QApplication.instance().setPalette(palette)
        self.setPalette(palette)

        base_qss = styles.get_stylesheet(current_mode)

        custom_qss = f"""
            {base_qss}

            QFrame#sidebarPanel {{
                background-color: {c['sidebar_bg']};
                border-right: 1px solid {c['card_border']};
            }}

            QFrame#appleCard {{
                background-color: {c['card_bg']};
                border: 1px solid {c['card_border']};
                border-radius: 10px;
            }}

            QPushButton[class="primaryButton"] {{
                background-color: {c['accent']};
                border: 1px solid {c['accent']};
                color: {c['accent_text']};
                font-weight: 600;
            }}

            QPushButton[class="primaryButton"]:hover {{
                background-color: {c['accent_hover']};
                border-color: {c['accent_hover']};
            }}
        """
        self.setStyleSheet(custom_qss)
