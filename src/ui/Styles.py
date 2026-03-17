def QWidget() -> str:
    return """
        QWidget {
            background-color: rgba(30, 30, 30, 220);
            border-radius: 8px;
           color: white;
        }

    """


def QProgressBar() -> str:
    return """
        QProgressBar {
            border: 1px solid #444;
            border-radius: 4px;
            background-color: #222;
            text-align: center;
            color: transparent;
        }
        QProgressBar::chunk {
            background-color: #9A67EA ;
            border-radius: 3px;
        }

    """


def QLabel() -> str:
    return """
        QLabel {
            color: #ffffff;
            font-family: sans-serif;
            font-weight: bold;
            font-size: 30px;
            background: transparent;
        }

    """


def QPushButton() -> str:
    return """
        QPushButton {
                background - color: transparent;
                border: 1px solid transparent;
                border-radius: 5px;
                padding: 20px;
                font-size: 14px;
                font-weight: bold;
                min-width: 120px;
        }
        QPushButton:hover {
                background - color: rgba(255, 255, 255, 0.1);
                border: 1px solid #5294e2;
        }

    """


def QScrollbar() -> str:
    return """
        QScrollBar:vertical {
            border: none;
            background-color: #1a1b26;
            width: 12px;
            margin: 0px;
        }
        
        QScrollBar::handle:vertical {
            background-color: #9d7cd8; 
            min-height: 30px;
            border-radius: 6px; 
        }
        
        QScrollBar::handle:vertical:hover {
            background-color: #bb9af7;
        }
        
        QScrollBar::sub-line:vertical, QScrollBar::add-line:vertical {
            height: 0px;
            background: none;
        }
        
        QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {
            background: transparent;
        }

    """


def QListWidget() -> str:
    return """
        QListWidget {
            border: none;
            outline: none;
            background-color: transparent;
        }
        QListWidget::item{
            border-radius: 8px;
            padding: 5px;
        }
        QListWidget::item:selected {
            background-color: #9A67EA;
        }

    """
