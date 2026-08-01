import subprocess
from PySide6.QtCore import Qt
from PySide6.QtWidgets import QApplication, QWidget, QSlider, QHBoxLayout, QLabel
from PySide6.QtGui import QKeyEvent
from infrastructure.display.brightnessctl_repository import BrightnessctlRepository
from application.display.set_brightness_use_case import SetBrightnessUseCase
import presentation.gui.styles as styles


class BrightnessPopup(QWidget):
    def __init__(self, mode: str = "dark"):
        super().__init__()
        self.use_case = SetBrightnessUseCase(BrightnessctlRepository())
        self.brilho = self.use_case.get_current()

        self.setWindowFlags(
            Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool
        )
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setFixedSize(320, 60)

        layout = QHBoxLayout()
        layout.setContentsMargins(16, 10, 16, 10)
        layout.setSpacing(12)

        icon_lbl = QLabel("☀️")
        icon_lbl.setStyleSheet("font-size: 16px; background: transparent;")

        self.slider = QSlider(Qt.Horizontal)
        self.slider.setRange(1, 100)
        self.slider.setValue(self.brilho)

        self.slider.valueChanged.connect(self.set_brightness)
        self.slider.sliderReleased.connect(self.close_app)

        layout.addWidget(icon_lbl)
        layout.addWidget(self.slider)
        self.setLayout(layout)

        c = styles.get_colors(mode)
        self.setStyleSheet(
            f"""
            QWidget {{
                background-color: {c['osd_bg']};
                border: 1px solid {c['osd_border']};
                border-radius: 16px;
            }}
            QSlider::groove:horizontal {{
                border: none;
                height: 10px;
                background: {c['progress_bg']};
                margin: 0px;
                border-radius: 5px;
            }}
            QSlider::handle:horizontal {{
                background: {c['accent']};
                border: 2px solid {c['window_bg']};
                width: 20px;
                height: 20px;
                margin: -5px 0;
                border-radius: 10px;
            }}
            QSlider::sub-page:horizontal {{
                background: {c['accent']};
                border-radius: 5px;
            }}
        """
        )

        self.center_on_screen()

    def center_on_screen(self):
        screen = QApplication.primaryScreen()
        if screen:
            rect = screen.availableGeometry()
            center_point = rect.center()
            frame_geom = self.frameGeometry()
            frame_geom.moveCenter(center_point)
            self.move(frame_geom.topLeft())

    def set_brightness(self, value):
        self.brilho = value
        self.use_case.repository.set_brightness(value)

    def set_brightness_ext_monitor(self, value):
        if value != 100:
            value -= 10
        if value <= 0:
            value = 1
        try:
            subprocess.run(
                ["ddcutil", "setvcp", "10", str(value), "--bus=6", "--noverify"],
                capture_output=True,
            )
        except Exception:
            pass

    def close_app(self):
        self.close()
        self.set_brightness_ext_monitor(self.brilho)

    def keyPressEvent(self, event: QKeyEvent, /) -> None:
        if event.key() == Qt.Key.Key_Escape:
            self.close()
        else:
            super().keyPressEvent(event)
