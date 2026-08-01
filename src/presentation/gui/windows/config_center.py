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
from domain.theme.entities import LightDMSettings, AppearanceSettings


class ConfigCenterWindow(QWidget):
    def __init__(self):
        super().__init__()
        QApplication.setDesktopFileName("sway-manager")
        self.setWindowTitle("SwayManager Control Center")
        self.resize(880, 600)
        self.setObjectName("mainWindow")

        # Instantiate Use Cases
        self.battery_use_case = ToggleBatteryConservationUseCase(SysfsBatteryRepository())
        self.idle_use_case = ToggleIdleUseCase(SwayIdleRepository())
        self.power_use_case = TogglePowerProfileUseCase(PowerProfilesRepository())
        self.theme_repo = GtkQtThemeRepository()
        self.theme_use_case = ToggleThemeUseCase(self.theme_repo)
        self.appearance_use_case = ManageAppearanceUseCase(self.theme_repo)
        self.wallpaper_use_case = SetWallpaperUseCase(SwayWallpaperRepository())
        self.lightdm_use_case = UpdateLightDMUseCase(LightDMRepository())

        self.pasta_imagens = os.path.expanduser("~/Imagens/Wallpapers")
        if not os.path.exists(self.pasta_imagens):
            self.pasta_imagens = os.path.expanduser("~/Pictures")

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
            ("🖼️  Papel de Parede", 0),
            ("🌓  Aparência & Tema", 1),
            ("🔋  Bateria & Energia", 2),
            ("☕  Suspensão & Idle", 3),
            ("🚦  Login & LightDM", 4),
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
            subprocess.run(
                ["notify-send", "Papel de Parede", f"Aplicado: {os.path.basename(caminho_imagem)}"]
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

        # Card 1: Background
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

        # Card 2: Greeter Styles Form
        card_style, style_layout = self.create_card(
            "Estilo e Aparência do Greeter",
            "Configure o tema GTK, tema de ícones, cursor e fonte para a tela de login do LightDM."
        )

        form_layout = QFormLayout()
        form_layout.setSpacing(12)

        options = self.appearance_use_case.get_available_options()

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

        self.txt_font_name = ThemeComboBox(
            options.fonts,
            self.lightdm_settings.get("font-name", "Sans 11"),
            editable=False,
        )

        self.txt_clock_format = QLineEdit(self.lightdm_settings.get("clock-format", "%a, %d %b %H:%M"))

        form_layout.addRow(QLabel("Tema GTK:"), self.txt_gtk_theme)
        form_layout.addRow(QLabel("Tema de Ícones:"), self.txt_icon_theme)
        form_layout.addRow(QLabel("Tema do Cursor:"), self.txt_cursor_theme)
        form_layout.addRow(QLabel("Fonte:"), self.txt_font_name)
        form_layout.addRow(QLabel("Formato do Relógio:"), self.txt_clock_format)

        style_layout.addLayout(form_layout)

        # Sync button row
        sync_row = QHBoxLayout()
        btn_sync_appearance = QPushButton("✨ Sincronizar Aparência do Sistema")
        btn_sync_appearance.clicked.connect(self.sincronizar_aparencia_lightdm)
        sync_row.addStretch()
        sync_row.addWidget(btn_sync_appearance)
        style_layout.addLayout(sync_row)

        layout.addWidget(card_style)

        # Card 3: Checkbox Options
        card_opt, opt_layout = self.create_card("Opções Adicionais de Login")

        self.chk_user_bg = QCheckBox("Carregar wallpaper individual do usuário (draw-user-backgrounds)")
        self.chk_user_bg.setChecked(self.lightdm_settings.get("draw-user-backgrounds", "false").lower() == "true")

        self.chk_hide_user_img = QCheckBox("Ocultar foto do usuário na tela de login (hide-user-image)")
        self.chk_hide_user_img.setChecked(self.lightdm_settings.get("hide-user-image", "false").lower() == "true")

        opt_layout.addWidget(self.chk_user_bg)
        opt_layout.addWidget(self.chk_hide_user_img)
        layout.addWidget(card_opt)

        # Action Bar at bottom
        action_layout = QHBoxLayout()
        self.lbl_lightdm_status = QLabel("")
        self.lbl_lightdm_status.setStyleSheet("font-size: 13px; font-weight: 600; color: #34C759;")

        btn_save_lightdm = QPushButton("💾 Salvar Configurações no LightDM")
        btn_save_lightdm.setProperty("class", "primaryButton")
        btn_save_lightdm.clicked.connect(self.salvar_config_lightdm)

        action_layout.addWidget(self.lbl_lightdm_status)
        action_layout.addStretch()
        action_layout.addWidget(btn_save_lightdm)
        layout.addLayout(action_layout)

        layout.addStretch()
        container.setLayout(layout)
        scroll.setWidget(container)
        return scroll

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

    def usar_wallpaper_sway_no_lightdm(self):
        target = self.wallpaper_use_case.get_current()
        if target:
            self.selected_lightdm_bg = target
            txt, tooltip = self.format_lightdm_bg_text("Selecionada (Sway)", target)
            self.lbl_lightdm_bg.setText(txt)
            self.lbl_lightdm_bg.setToolTip(tooltip)
            nome_trunc = StringUtils.truncar_nome_arquivo(os.path.basename(target), max_len=25)
            self.lbl_lightdm_status.setText(f"✅ Wallpaper do Sway selecionado: {nome_trunc}")
            self.lbl_lightdm_status.setToolTip(target)
        else:
            self.lbl_lightdm_status.setText("⚠️ Nenhum wallpaper atual do Sway foi encontrado.")

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

        self.lbl_lightdm_status.setText("✨ Tema, ícones, cursor e fonte do sistema sincronizados no formulário!")

    def salvar_config_lightdm(self):
        dict_settings = {
            "theme-name": self.txt_gtk_theme.currentText().strip(),
            "icon-theme-name": self.txt_icon_theme.currentText().strip(),
            "cursor-theme-name": self.txt_cursor_theme.currentText().strip(),
            "font-name": self.txt_font_name.currentText().strip(),
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
            bg_path = self.lightdm_settings.get("background", "Nenhum")
            txt, tooltip = self.format_lightdm_bg_text("Imagem Atual", bg_path)
            self.lbl_lightdm_bg.setText(txt)
            self.lbl_lightdm_bg.setToolTip(tooltip)
            self.selected_lightdm_bg = None
        else:
            self.lbl_lightdm_status.setText("❌ Erro ao salvar (operação cancelada ou falha).")

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
