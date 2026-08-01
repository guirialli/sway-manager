import subprocess
from PySide6.QtCore import Qt
from PySide6.QtWidgets import QApplication, QWidget, QSlider, QHBoxLayout
from PySide6.QtGui import QKeyEvent
from infrastructure.display.brightnessctl_repository import BrightnessctlRepository
from application.display.set_brightness_use_case import SetBrightnessUseCase


class BrightnessPopup(QWidget):
    def __init__(self):
        super().__init__()
        self.use_case = SetBrightnessUseCase(BrightnessctlRepository())
        self.brilho = self.use_case.get_current()

        self.setWindowFlags(
            Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool
        )
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setFixedSize(320, 60)

        layout = QHBoxLayout()
        layout.setContentsMargins(15, 10, 15, 10)

        self.slider = QSlider(Qt.Horizontal)
        self.slider.setRange(1, 100)
        self.slider.setValue(self.brilho)

        self.slider.valueChanged.connect(self.set_brightness)
        self.slider.sliderReleased.connect(self.close_app)

        layout.addWidget(self.slider)
        self.setLayout(layout)

        self.setStyleSheet(
            """
            QWidget {
                background-color: #1e1e2e;
                border: 2px solid #89b4fa;
                border-radius: 15px;
            }
            QSlider::groove:horizontal {
                border: 1px solid #313244;
                height: 8px;
                background: #45475a;
                margin: 2px 0;
                border-radius: 4px;
            }
            QSlider::handle:horizontal {
                background: #89b4fa;
                border: 1px solid #89b4fa;
                width: 18px;
                height: 18px;
                margin: -5px 0;
                border-radius: 9px;
            }
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
