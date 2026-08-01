"""
Apple macOS Human Interface Guidelines (HIG) Styling System for SwayManager
Provides unified dark and light mode stylesheet definitions for PySide6 components.
"""

FONT_FAMILY = "-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'SF Pro Display', 'Segoe UI', Roboto, sans-serif"

DARK_COLORS = {
    "window_bg": "#1E1E1E",
    "sidebar_bg": "#252528",
    "card_bg": "#2C2C2E",
    "card_border": "rgba(255, 255, 255, 0.08)",
    "text_primary": "#FFFFFF",
    "text_secondary": "#8E8E93",
    "text_muted": "#636366",
    "accent": "#007AFF",
    "accent_hover": "#0062CC",
    "accent_text": "#FFFFFF",
    "input_bg": "#1C1C1E",
    "input_border": "rgba(255, 255, 255, 0.15)",
    "button_bg": "rgba(255, 255, 255, 0.08)",
    "button_hover": "rgba(255, 255, 255, 0.15)",
    "button_border": "rgba(255, 255, 255, 0.1)",
    "selection_bg": "#007AFF",
    "scrollbar_handle": "rgba(255, 255, 255, 0.25)",
    "scrollbar_handle_hover": "rgba(255, 255, 255, 0.4)",
    "osd_bg": "rgba(30, 30, 32, 220)",
    "osd_border": "rgba(255, 255, 255, 0.15)",
    "progress_bg": "rgba(255, 255, 255, 0.15)",
    "progress_chunk": "#007AFF",
}

LIGHT_COLORS = {
    "window_bg": "#F5F5F7",
    "sidebar_bg": "#E8E8ED",
    "card_bg": "#FFFFFF",
    "card_border": "rgba(0, 0, 0, 0.08)",
    "text_primary": "#000000",
    "text_secondary": "#636366",
    "text_muted": "#8E8E93",
    "accent": "#007AFF",
    "accent_hover": "#0062CC",
    "accent_text": "#FFFFFF",
    "input_bg": "#FFFFFF",
    "input_border": "rgba(0, 0, 0, 0.15)",
    "button_bg": "rgba(0, 0, 0, 0.04)",
    "button_hover": "rgba(0, 0, 0, 0.08)",
    "button_border": "rgba(0, 0, 0, 0.1)",
    "selection_bg": "#007AFF",
    "scrollbar_handle": "rgba(0, 0, 0, 0.2)",
    "scrollbar_handle_hover": "rgba(0, 0, 0, 0.35)",
    "osd_bg": "rgba(240, 240, 245, 220)",
    "osd_border": "rgba(0, 0, 0, 0.12)",
    "progress_bg": "rgba(0, 0, 0, 0.1)",
    "progress_chunk": "#007AFF",
}


from PySide6.QtGui import QPalette, QColor


def get_colors(mode: str = "dark") -> dict:
    return DARK_COLORS if mode.lower() == "dark" else LIGHT_COLORS


def get_palette(mode: str = "dark") -> QPalette:
    c = get_colors(mode)
    palette = QPalette()
    bg_color = QColor(c["window_bg"])
    card_bg = QColor(c["card_bg"])
    input_bg = QColor(c["input_bg"])
    text_primary = QColor(c["text_primary"])
    text_secondary = QColor(c["text_secondary"])
    accent = QColor(c["accent"])
    accent_text = QColor(c["accent_text"])

    palette.setColor(QPalette.Window, bg_color)
    palette.setColor(QPalette.WindowText, text_primary)
    palette.setColor(QPalette.Base, input_bg)
    palette.setColor(QPalette.AlternateBase, card_bg)
    palette.setColor(QPalette.ToolTipBase, card_bg)
    palette.setColor(QPalette.ToolTipText, text_primary)
    palette.setColor(QPalette.Text, text_primary)
    palette.setColor(QPalette.PlaceholderText, text_secondary)
    palette.setColor(QPalette.Button, card_bg)
    palette.setColor(QPalette.ButtonText, text_primary)
    palette.setColor(QPalette.Highlight, accent)
    palette.setColor(QPalette.HighlightedText, accent_text)
    palette.setColor(QPalette.BrightText, accent_text)

    return palette



def get_stylesheet(mode: str = "dark") -> str:
    c = get_colors(mode)
    return f"""
        * {{
            font-family: {FONT_FAMILY};
            font-size: 13px;
            color: {c['text_primary']};
        }}

        QWidget#mainWindow, QWidget#mainContainer {{
            background-color: {c['window_bg']};
        }}

        /* Apple Inset Grouped Cards */
        QFrame.appleCard {{
            background-color: {c['card_bg']};
            border: 1px solid {c['card_border']};
            border-radius: 10px;
        }}

        /* Section Titles (macOS style uppercase muted subheader) */
        QLabel.sectionHeader {{
            font-size: 11px;
            font-weight: 600;
            color: {c['text_secondary']};
            text-transform: uppercase;
            letter-spacing: 0.5px;
            background: transparent;
        }}

        QLabel.cardTitle {{
            font-size: 13px;
            font-weight: 600;
            color: {c['text_primary']};
            background: transparent;
        }}

        QLabel.secondaryText {{
            font-size: 12px;
            color: {c['text_secondary']};
            background: transparent;
        }}

        /* Apple Primary & Secondary Buttons */
        QPushButton {{
            background-color: {c['button_bg']};
            border: 1px solid {c['button_border']};
            border-radius: 6px;
            padding: 6px 14px;
            color: {c['text_primary']};
            font-weight: 500;
            font-size: 13px;
            min-height: 22px;
        }}

        QPushButton:hover {{
            background-color: {c['button_hover']};
        }}

        QPushButton:pressed {{
            background-color: {c['button_bg']};
        }}

        QPushButton.primaryButton {{
            background-color: {c['accent']};
            border: 1px solid {c['accent']};
            color: {c['accent_text']};
            font-weight: 600;
        }}

        QPushButton.primaryButton:hover {{
            background-color: {c['accent_hover']};
            border-color: {c['accent_hover']};
        }}

        /* Apple Input Controls */
        QLineEdit {{
            background-color: {c['input_bg']};
            border: 1px solid {c['input_border']};
            border-radius: 6px;
            padding: 6px 10px;
            color: {c['text_primary']};
            font-size: 13px;
        }}

        QLineEdit:focus {{
            border: 1.5px solid {c['accent']};
        }}

        /* QComboBox Styling */
        QComboBox {{
            background-color: {c['input_bg']};
            border: 1px solid {c['input_border']};
            border-radius: 6px;
            padding: 6px 10px;
            color: {c['text_primary']};
            font-size: 13px;
            min-height: 20px;
            selection-background-color: {c['accent']};
            selection-color: {c['accent_text']};
        }}

        QComboBox:hover {{
            border-color: {c['accent']};
        }}

        QComboBox:on, QComboBox:focus {{
            border-color: {c['accent']};
            background-color: {c['input_bg']};
            color: {c['text_primary']};
        }}

        QComboBox QLineEdit {{
            background-color: transparent;
            border: none;
            color: {c['text_primary']};
            font-size: 13px;
            selection-background-color: {c['accent']};
            selection-color: {c['accent_text']};
        }}

        QComboBox::drop-down {{
            subcontrol-origin: padding;
            subcontrol-position: top right;
            width: 24px;
            border-left: none;
            border-top-right-radius: 6px;
            border-bottom-right-radius: 6px;
        }}

        /* QComboBox Popup Dropdown List */
        QComboBox QAbstractItemView, QComboBox QListView {{
            background-color: {c['card_bg']};
            border: 1px solid {c['card_border']};
            border-radius: 8px;
            padding: 4px;
            color: {c['text_primary']};
            selection-background-color: {c['accent']};
            selection-color: {c['accent_text']};
            outline: none;
        }}

        QComboBox QAbstractItemView::viewport, QComboBox QListView::viewport {{
            background-color: {c['card_bg']};
            color: {c['text_primary']};
        }}

        QComboBox QAbstractItemView::item, QComboBox QListView::item {{
            min-height: 26px;
            padding: 4px 8px;
            border-radius: 4px;
            color: {c['text_primary']};
            background-color: {c['card_bg']};
        }}

        QComboBox QAbstractItemView::item:hover, QComboBox QListView::item:hover {{
            background-color: {c['button_hover']};
            color: {c['text_primary']};
        }}

        QComboBox QAbstractItemView::item:selected, QComboBox QListView::item:selected {{
            background-color: {c['accent']};
            color: {c['accent_text']};
        }}

        /* Checkbox & Switch */
        QCheckBox {{
            color: {c['text_primary']};
            font-size: 13px;
            spacing: 8px;
            background: transparent;
        }}

        QCheckBox::indicator {{
            width: 18px;
            height: 18px;
            border-radius: 4px;
            border: 1px solid {c['input_border']};
            background-color: {c['input_bg']};
        }}

        QCheckBox::indicator:checked {{
            background-color: {c['accent']};
            border-color: {c['accent']};
            image: url(none);
        }}

        /* General QListWidget & Gallery Grid */
        QListWidget {{
            background-color: transparent;
            border: none;
            outline: none;
        }}

        QListWidget::viewport {{
            background-color: transparent;
        }}

        QListWidget::item {{
            border-radius: 8px;
            padding: 6px;
            background-color: transparent;
            color: {c['text_primary']};
        }}

        QListWidget::item:hover {{
            background-color: {c['button_hover']};
        }}

        QListWidget::item:selected {{
            background-color: {c['accent']};
            color: {c['accent_text']};
        }}

        /* macOS Sidebar Style */
        QListWidget#sidebar {{
            background-color: {c['sidebar_bg']};
            border: none;
            outline: none;
            padding: 10px 6px;
        }}

        QListWidget#sidebar::item {{
            border-radius: 6px;
            padding: 8px 12px;
            margin-bottom: 2px;
            color: {c['text_primary']};
            font-size: 13px;
            font-weight: 500;
        }}

        QListWidget#sidebar::item:hover {{
            background-color: {c['button_hover']};
        }}

        QListWidget#sidebar::item:selected {{
            background-color: {c['accent']};
            color: {c['accent_text']};
            font-weight: 600;
        }}

        /* Scrollbars (macOS Overlay Style) */
        QScrollBar:vertical {{
            border: none;
            background: transparent;
            width: 8px;
            margin: 0px;
        }}

        QScrollBar::handle:vertical {{
            background-color: {c['scrollbar_handle']};
            min-height: 30px;
            border-radius: 4px;
        }}

        QScrollBar::handle:vertical:hover {{
            background-color: {c['scrollbar_handle_hover']};
        }}

        QScrollBar::sub-line:vertical, QScrollBar::add-line:vertical {{
            height: 0px;
            background: none;
        }}

        QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {{
            background: transparent;
        }}

        QScrollBar:horizontal {{
            border: none;
            background: transparent;
            height: 8px;
            margin: 0px;
        }}

        QScrollBar::handle:horizontal {{
            background-color: {c['scrollbar_handle']};
            min-width: 30px;
            border-radius: 4px;
        }}

        QScrollBar::handle:horizontal:hover {{
            background-color: {c['scrollbar_handle_hover']};
        }}

        QScrollBar::sub-line:horizontal, QScrollBar::add-line:horizontal {{
            width: 0px;
            background: none;
        }}

        QScrollBar::add-page:horizontal, QScrollBar::sub-page:horizontal {{
            background: transparent;
        }}
    """


# Backward compatibility functions
def QWidget(mode: str = "dark") -> str:
    c = get_colors(mode)
    return f"""
        QWidget {{
            background-color: {c['osd_bg']};
            border-radius: 14px;
            border: 1px solid {c['osd_border']};
            color: {c['text_primary']};
            font-family: {FONT_FAMILY};
        }}
    """


def QProgressBar(mode: str = "dark") -> str:
    c = get_colors(mode)
    return f"""
        QProgressBar {{
            border: none;
            border-radius: 4px;
            background-color: {c['progress_bg']};
            text-align: center;
            color: transparent;
        }}
        QProgressBar::chunk {{
            background-color: {c['progress_chunk']};
            border-radius: 4px;
        }}
    """


def QLabel(mode: str = "dark") -> str:
    c = get_colors(mode)
    return f"""
        QLabel {{
            color: {c['text_primary']};
            font-family: {FONT_FAMILY};
            font-weight: 600;
            font-size: 18px;
            background: transparent;
        }}
    """


def QPushButton(mode: str = "dark") -> str:
    c = get_colors(mode)
    return f"""
        QPushButton {{
            background-color: {c['button_bg']};
            border: 1px solid {c['button_border']};
            border-radius: 8px;
            padding: 12px 18px;
            font-size: 13px;
            font-weight: 600;
            color: {c['text_primary']};
            font-family: {FONT_FAMILY};
        }}
        QPushButton:hover {{
            background-color: {c['button_hover']};
        }}
        QPushButton:pressed {{
            background-color: {c['accent']};
            color: {c['accent_text']};
        }}
    """


def QScrollbar(mode: str = "dark") -> str:
    c = get_colors(mode)
    return f"""
        QScrollBar:vertical {{
            border: none;
            background: transparent;
            width: 8px;
        }}
        QScrollBar::handle:vertical {{
            background-color: {c['scrollbar_handle']};
            min-height: 30px;
            border-radius: 4px;
        }}
        QScrollBar::handle:vertical:hover {{
            background-color: {c['scrollbar_handle_hover']};
        }}
        QScrollBar::sub-line:vertical, QScrollBar::add-line:vertical {{
            height: 0px;
            background: none;
        }}
        QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {{
            background: transparent;
        }}
    """


def QListWidget(mode: str = "dark") -> str:
    c = get_colors(mode)
    return f"""
        QListWidget {{
            border: none;
            outline: none;
            background-color: transparent;
        }}
        QListWidget::item {{
            border-radius: 8px;
            padding: 6px;
        }}
        QListWidget::item:selected {{
            background-color: {c['accent']};
            color: {c['accent_text']};
        }}
    """
