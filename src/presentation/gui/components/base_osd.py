from PySide6.QtCore import Qt, QTimer
from PySide6.QtWidgets import QWidget, QApplication, QHBoxLayout, QProgressBar, QLabel
import presentation.gui.styles as styles


class OSD(QWidget):
    def __init__(self, percent: int, label: str):
        super().__init__()
        self.setWindowFlag(
            Qt.WindowType.FramelessWindowHint | Qt.WindowType.WindowStaysOnTopHint
        )

        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setFixedSize(300, 56)

        QApplication.setDesktopFileName("sway.osd.brightness")
        layout = QHBoxLayout()
        layout.setContentsMargins(16, 10, 20, 10)
        layout.setSpacing(12)
        self.setLayout(layout)

        self.setStyleSheet(f"""
            {styles.QWidget()}
            {styles.QProgressBar()}
            {styles.QLabel()}
        """)

        icon_label = QLabel(label)
        icon_label.setFixedWidth(40)
        icon_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(icon_label)

        self.bar = QProgressBar()
        self.bar.setRange(0, 100)
        self.bar.setValue(percent)
        self.bar.setTextVisible(False)
        self.bar.setFixedHeight(10)
        layout.addWidget(self.bar)

        QTimer.singleShot(1200, self.close)
