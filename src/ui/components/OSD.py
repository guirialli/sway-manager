from PySide6.QtCore import Qt, QTimer
from PySide6.QtWidgets import QWidget, QApplication, QHBoxLayout, QProgressBar, QLabel
import ui.Styles as styles


class OSD(QWidget):
    def __init__(self, percent: int, label: str):
        super().__init__()
        self.setWindowFlag(
            Qt.WindowType.FramelessWindowHint | Qt.WindowType.WindowStaysOnTopHint
        )

        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.resize(300, 50)

        QApplication.setDesktopFileName("sway.osd.brightness")
        layout = QHBoxLayout()
        self.setLayout(layout)

        self.setStyleSheet(f"""
            {styles.QWidget()}
            {styles.QProgressBar()}
            {styles.QLabel()}
        """)

        icon_label = QLabel(label)
        icon_label.setFixedWidth(50)
        icon_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(icon_label)

        self.bar = QProgressBar()
        self.bar.setRange(0, 100)
        self.bar.setValue(percent)
        self.bar.setTextVisible(False)
        layout.addWidget(self.bar)

        QTimer.singleShot(1000, QApplication.quit)
