"""Frozen-screen area selector used by the screenshot command.

The selector receives one frozen image per Sway output.  It keeps interaction in
logical desktop coordinates, then maps the final rectangle back to the physical
pixels of each output when writing the image.
"""

from __future__ import annotations

import math
import subprocess
import sys
from dataclasses import dataclass
from typing import Iterable, Optional

from PySide6.QtCore import QEventLoop, QPoint, QRect, Qt
from PySide6.QtGui import QColor, QImage, QPainter, QPen, QPixmap, QScreen
from PySide6.QtWidgets import QApplication, QWidget


MIN_SELECTION_SIZE = 6


class OverlayUnavailableError(RuntimeError):
    """Raised when no complete frozen snapshot is available for the desktop."""


@dataclass(frozen=True)
class FrozenScreen:
    """A frozen physical-pixel image and its logical desktop geometry."""

    name: str
    geometry: QRect
    image: QImage

    @property
    def scale_x(self) -> float:
        return self.image.width() / self.geometry.width()

    @property
    def scale_y(self) -> float:
        return self.image.height() / self.geometry.height()


class SelectionState:
    """State machine for one mouse drag in global logical coordinates."""

    def __init__(self) -> None:
        self.start: Optional[QPoint] = None
        self.current: Optional[QPoint] = None

    @property
    def rectangle(self) -> Optional[QRect]:
        if self.start is None or self.current is None:
            return None
        return QRect(
            min(self.start.x(), self.current.x()),
            min(self.start.y(), self.current.y()),
            abs(self.current.x() - self.start.x()) + 1,
            abs(self.current.y() - self.start.y()) + 1,
        )

    def begin(self, position: QPoint) -> None:
        self.start = QPoint(position)
        self.current = QPoint(position)

    def update(self, position: QPoint) -> None:
        if self.start is not None:
            self.current = QPoint(position)

    def finish(self, position: QPoint) -> Optional[QRect]:
        self.update(position)
        rectangle = self.rectangle
        if (
            rectangle is None
            or rectangle.width() < MIN_SELECTION_SIZE
            or rectangle.height() < MIN_SELECTION_SIZE
        ):
            return None
        return rectangle


def load_frozen_screens(
    screens: Iterable[QScreen], freeze_files: dict[str, str]
) -> list[FrozenScreen]:
    """Load a complete frozen snapshot for every Qt screen.

    A partial snapshot is unsafe: the user could select an output that has no
    matching image.  In that case the repository falls back to ``slurp``.
    """

    frozen_screens: list[FrozenScreen] = []
    for screen in screens:
        path = freeze_files.get(screen.name())
        image = QImage(path) if path else QImage()
        if image.isNull() or screen.geometry().isEmpty():
            raise OverlayUnavailableError(
                f"Missing frozen screenshot for output {screen.name()}"
            )
        frozen_screens.append(FrozenScreen(screen.name(), screen.geometry(), image))
    if not frozen_screens:
        raise OverlayUnavailableError("No screen is available for selection")
    return frozen_screens


def compose_selection(
    selection: QRect, frozen_screens: Iterable[FrozenScreen]
) -> Optional[QImage]:
    """Compose a global logical selection at the highest participating scale."""

    intersections = [
        (screen, selection.intersected(screen.geometry))
        for screen in frozen_screens
        if not selection.intersected(screen.geometry).isEmpty()
    ]
    if not intersections:
        return None

    output_scale = max(
        max(screen.scale_x, screen.scale_y) for screen, _ in intersections
    )
    canvas = QImage(
        math.ceil(selection.width() * output_scale),
        math.ceil(selection.height() * output_scale),
        QImage.Format_ARGB32,
    )
    canvas.fill(Qt.transparent)

    painter = QPainter(canvas)
    painter.setRenderHint(QPainter.SmoothPixmapTransform, True)
    for screen, intersection in intersections:
        local_x = intersection.x() - screen.geometry.x()
        local_y = intersection.y() - screen.geometry.y()
        source_x = round(local_x * screen.scale_x)
        source_y = round(local_y * screen.scale_y)
        source_right = round((local_x + intersection.width()) * screen.scale_x)
        source_bottom = round((local_y + intersection.height()) * screen.scale_y)
        destination_x = round((intersection.x() - selection.x()) * output_scale)
        destination_y = round((intersection.y() - selection.y()) * output_scale)
        destination_right = round(
            (intersection.x() - selection.x() + intersection.width()) * output_scale
        )
        destination_bottom = round(
            (intersection.y() - selection.y() + intersection.height()) * output_scale
        )
        source = QRect(
            source_x,
            source_y,
            source_right - source_x,
            source_bottom - source_y,
        )
        destination = QRect(
            destination_x,
            destination_y,
            destination_right - destination_x,
            destination_bottom - destination_y,
        )
        painter.drawImage(destination, screen.image.copy(source))
    painter.end()
    return canvas


class SingleScreenOverlayWindow(QWidget):
    """One transparent input window shown over a physical output."""

    def __init__(
        self,
        screen: QScreen,
        frozen_image: QImage,
        controller: "MultiMonitorSelectionController",
    ) -> None:
        super().__init__()
        self.screen_obj = screen
        self.screen_geometry = screen.geometry()
        self.controller = controller
        self.pixmap = QPixmap.fromImage(frozen_image)

        self.setWindowTitle(f"SwayManagerFreeze_{screen.name()}")
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool)
        self.setAttribute(Qt.WA_DeleteOnClose, True)
        self.setGeometry(self.screen_geometry)
        self.setCursor(Qt.CrossCursor)
        self.setFocusPolicy(Qt.StrongFocus)
        self.setMouseTracking(True)

    def paintEvent(self, _event) -> None:
        painter = QPainter(self)
        painter.drawPixmap(self.rect(), self.pixmap)

        selection = self.controller.selection.rectangle
        if selection is None:
            painter.fillRect(self.rect(), QColor(0, 0, 0, 115))
            return

        intersection = selection.intersected(self.screen_geometry)
        if intersection.isEmpty():
            painter.fillRect(self.rect(), QColor(0, 0, 0, 115))
            return

        x = intersection.x() - self.screen_geometry.x()
        y = intersection.y() - self.screen_geometry.y()
        width = intersection.width()
        height = intersection.height()
        mask = QColor(0, 0, 0, 115)
        painter.fillRect(0, 0, self.width(), y, mask)
        painter.fillRect(0, y, x, height, mask)
        painter.fillRect(x + width, y, self.width() - x - width, height, mask)
        painter.fillRect(0, y + height, self.width(), self.height() - y - height, mask)

        painter.setPen(QPen(QColor("#007AFF"), 2))
        painter.drawRect(x, y, width, height)
        self._draw_size_label(painter, selection, x, y)

    def _draw_size_label(
        self, painter: QPainter, selection: QRect, x: int, y: int
    ) -> None:
        label = f"{selection.width()} × {selection.height()} px"
        metrics = painter.fontMetrics()
        label_width = metrics.horizontalAdvance(label) + 16
        label_height = metrics.height() + 8
        label_y = y - label_height - 6
        if label_y < 0:
            label_y = y + 6
        painter.fillRect(x, label_y, label_width, label_height, QColor(28, 28, 32, 230))
        painter.setPen(Qt.white)
        painter.drawText(
            QRect(x, label_y, label_width, label_height), Qt.AlignCenter, label
        )

    def mousePressEvent(self, event) -> None:
        if event.button() == Qt.LeftButton:
            self.controller.begin_selection(self.mapToGlobal(event.position().toPoint()))
        elif event.button() == Qt.RightButton:
            self.controller.cancel_selection()

    def mouseMoveEvent(self, event) -> None:
        self.controller.update_selection(self.mapToGlobal(event.position().toPoint()))

    def mouseReleaseEvent(self, event) -> None:
        if event.button() == Qt.LeftButton:
            self.controller.complete_selection(self.mapToGlobal(event.position().toPoint()))

    def keyPressEvent(self, event) -> None:
        if event.key() == Qt.Key_Escape:
            self.controller.cancel_selection()

    def closeEvent(self, event) -> None:
        if not self.controller.finished:
            self.controller.cancel_selection()
        event.accept()


class MultiMonitorSelectionController:
    """Coordinates overlay windows and saves the selected frozen pixels."""

    def __init__(self, freeze_files: dict[str, str]) -> None:
        application = QApplication.instance()
        self.app = application if application is not None else QApplication(sys.argv)
        self.frozen_screens = load_frozen_screens(self.app.screens(), freeze_files)
        self.selection = SelectionState()
        self.selected_rect: Optional[QRect] = None
        self.cancelled = False
        self.finished = False
        self.loop: Optional[QEventLoop] = None
        self.windows = [
            SingleScreenOverlayWindow(
                screen,
                next(item.image for item in self.frozen_screens if item.name == screen.name()),
                self,
            )
            for screen in self.app.screens()
        ]

    def begin_selection(self, position: QPoint) -> None:
        self.selection.begin(position)
        self._update_windows()

    def update_selection(self, position: QPoint) -> None:
        self.selection.update(position)
        self._update_windows()

    def complete_selection(self, position: QPoint) -> None:
        selection = self.selection.finish(position)
        if selection is None:
            self.cancel_selection()
            return
        self.selected_rect = selection
        self.close_all()

    def cancel_selection(self) -> None:
        self.cancelled = True
        self.selected_rect = None
        self.close_all()

    def _update_windows(self) -> None:
        for window in self.windows:
            window.update()

    def close_all(self) -> None:
        if self.finished:
            return
        self.finished = True
        for window in self.windows:
            window.close()
        if self.loop is not None and self.loop.isRunning():
            self.loop.quit()

    def save_cropped_selection(self, destination: str) -> bool:
        if self.selected_rect is None:
            return False
        image = compose_selection(self.selected_rect, self.frozen_screens)
        return image is not None and image.save(destination)

    def _place_windows_on_outputs(self) -> None:
        """Ask Sway to keep each popup on the output it represents."""

        for window in self.windows:
            command = (
                f'[title="{window.windowTitle()}"] move window to output '
                f'"{window.screen_obj.name()}", floating enable, fullscreen enable, focus'
            )
            try:
                subprocess.run(["swaymsg", command], capture_output=True, check=False)
            except OSError:
                # The Qt geometry is still enough on compositors without swaymsg.
                continue

    def exec_selection(self, destination: str) -> bool:
        for window in self.windows:
            window.show()
            window.raise_()
        self.app.processEvents()
        self._place_windows_on_outputs()
        if self.windows:
            self.windows[-1].activateWindow()

        self.loop = QEventLoop()
        self.loop.exec()
        return not self.cancelled and self.save_cropped_selection(destination)


class FreezeSelectionOverlay:
    @classmethod
    def select_area(cls, freeze_files: dict[str, str], dst_path: str) -> bool:
        return MultiMonitorSelectionController(freeze_files).exec_selection(dst_path)
