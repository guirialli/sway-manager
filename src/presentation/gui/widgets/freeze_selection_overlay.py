import sys
import time
import subprocess
from typing import Optional
from PySide6.QtWidgets import QWidget, QApplication
from PySide6.QtCore import Qt, QRect, QPoint, QEventLoop
from PySide6.QtGui import QPainter, QColor, QPen, QPixmap, QImage, QScreen


class SingleScreenOverlayWindow(QWidget):
    def __init__(
        self,
        screen: QScreen,
        screen_pixmap: QPixmap,
        controller: "MultiMonitorSelectionController",
    ):
        super().__init__()
        self.screen_obj = screen
        self.screen_pixmap = screen_pixmap
        self.controller = controller
        self.screen_geom = screen.geometry()

        self.setWindowTitle(f"SwayManagerFreeze_{screen.name()}")
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool)
        self.setAttribute(Qt.WA_DeleteOnClose, True)
        self.setProperty("app_id", "sway-manager-freeze")
        self.setGeometry(self.screen_geom)
        self.setCursor(Qt.CrossCursor)
        self.setMouseTracking(True)

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.drawPixmap(0, 0, self.width(), self.height(), self.screen_pixmap)

        mask_color = QColor(0, 0, 0, 115)

        start_g = self.controller.start_global
        curr_g = self.controller.current_global

        if start_g and curr_g:
            global_rect = QRect(start_g, curr_g).normalized()
            inter = global_rect.intersected(self.screen_geom)

            w, h = self.width(), self.height()

            if inter.isValid() and not inter.isEmpty():
                rx = inter.x() - self.screen_geom.x()
                ry = inter.y() - self.screen_geom.y()
                rw = inter.width()
                rh = inter.height()

                painter.fillRect(0, 0, w, ry, mask_color)
                painter.fillRect(0, ry, rx, rh, mask_color)
                painter.fillRect(rx + rw, ry, w - (rx + rw), rh, mask_color)
                painter.fillRect(0, ry + rh, w, h - (ry + rh), mask_color)

                pen = QPen(QColor(53, 132, 228), 2)
                painter.setPen(pen)
                painter.drawRect(rx, ry, rw, rh)

                if (
                    global_rect.contains(inter.topLeft())
                    and global_rect.width() > 10
                    and global_rect.height() > 10
                ):
                    text = f"{global_rect.width()} × {global_rect.height()} px"
                    fm = painter.fontMetrics()
                    tw = fm.horizontalAdvance(text) + 16
                    th = fm.height() + 8

                    bx = rx
                    by = ry - th - 6
                    if by < 0:
                        by = ry + 6

                    painter.fillRect(bx, by, tw, th, QColor(30, 30, 30, 220))
                    painter.setPen(QPen(QColor(255, 255, 255)))
                    painter.drawText(QRect(bx, by, tw, th), Qt.AlignCenter, text)
            else:
                painter.fillRect(0, 0, w, h, mask_color)
        else:
            painter.fillRect(0, 0, self.width(), self.height(), mask_color)

    def mousePressEvent(self, event):
        if event.button() == Qt.LeftButton:
            global_pos = self.mapToGlobal(event.pos())
            self.controller.start_selection(global_pos)
        elif event.button() == Qt.RightButton:
            self.controller.cancel_selection()

    def mouseMoveEvent(self, event):
        if self.controller.start_global:
            global_pos = self.mapToGlobal(event.pos())
            self.controller.update_selection(global_pos)

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.LeftButton:
            global_pos = self.mapToGlobal(event.pos())
            self.controller.finish_selection(global_pos)

    def keyPressEvent(self, event):
        if event.key() == Qt.Key_Escape:
            self.controller.cancel_selection()


class MultiMonitorSelectionController:
    def __init__(self, freeze_files: dict[str, str]):
        self.app = QApplication.instance() or QApplication(sys.argv)
        self.freeze_files = freeze_files
        self.start_global: Optional[QPoint] = None
        self.current_global: Optional[QPoint] = None
        self.selected_rect: Optional[QRect] = None
        self.cancelled = False
        self.windows: list[SingleScreenOverlayWindow] = []
        self.loop: Optional[QEventLoop] = None

        screens = self.app.screens()
        for s in screens:
            s_name = s.name()
            fpath = self.freeze_files.get(s_name)
            if fpath and not QImage(fpath).isNull():
                pixmap = QPixmap(fpath)
            else:
                pixmap = QPixmap(s.geometry().size())
                pixmap.fill(QColor("black"))

            win = SingleScreenOverlayWindow(s, pixmap, self)
            self.windows.append(win)

    def start_selection(self, global_pos: QPoint):
        self.start_global = global_pos
        self.current_global = global_pos
        self.update_all()

    def update_selection(self, global_pos: QPoint):
        self.current_global = global_pos
        self.update_all()

    def finish_selection(self, global_pos: QPoint):
        self.current_global = global_pos
        if self.start_global and self.current_global:
            r = QRect(self.start_global, self.current_global).normalized()
            if r.width() > 5 and r.height() > 5:
                self.selected_rect = r
            else:
                self.cancelled = True
        self.close_all()

    def cancel_selection(self):
        self.cancelled = True
        self.selected_rect = None
        self.close_all()

    def update_all(self):
        for w in self.windows:
            w.update()

    def close_all(self):
        for w in list(self.windows):
            try:
                w.hide()
                w.close()
            except Exception:
                pass
        if self.loop and self.loop.isRunning():
            self.loop.quit()
        if self.app:
            self.app.quit()

    def save_cropped_selection(self, dst_path: str) -> bool:
        if not self.selected_rect:
            return False

        global_rect = self.selected_rect
        screens = self.app.screens()

        intersecting = []
        for s in screens:
            s_geom = s.geometry()
            inter = global_rect.intersected(s_geom)
            if inter.isValid() and not inter.isEmpty():
                intersecting.append((s, s_geom, inter))

        if not intersecting:
            return False

        if len(intersecting) == 1:
            s, s_geom, inter = intersecting[0]
            fpath = self.freeze_files.get(s.name())
            if not fpath:
                return False

            img = QImage(fpath)
            if img.isNull():
                return False

            scale_x = img.width() / s_geom.width()
            scale_y = img.height() / s_geom.height()

            local_x = inter.x() - s_geom.x()
            local_y = inter.y() - s_geom.y()
            phys_x = int(round(local_x * scale_x))
            phys_y = int(round(local_y * scale_y))
            phys_w = int(round(inter.width() * scale_x))
            phys_h = int(round(inter.height() * scale_y))

            cropped = img.copy(QRect(phys_x, phys_y, phys_w, phys_h))
            return cropped.save(dst_path)
        else:
            canvas_w = global_rect.width()
            canvas_h = global_rect.height()
            canvas = QImage(canvas_w, canvas_h, QImage.Format_ARGB32)
            canvas.fill(QColor(0, 0, 0, 0))
            painter = QPainter(canvas)

            for s, s_geom, inter in intersecting:
                fpath = self.freeze_files.get(s.name())
                if not fpath:
                    continue
                img = QImage(fpath)
                if img.isNull():
                    continue

                scale_x = img.width() / s_geom.width()
                scale_y = img.height() / s_geom.height()

                local_x = inter.x() - s_geom.x()
                local_y = inter.y() - s_geom.y()
                phys_x = int(round(local_x * scale_x))
                phys_y = int(round(local_y * scale_y))
                phys_w = int(round(inter.width() * scale_x))
                phys_h = int(round(inter.height() * scale_y))

                cropped_slice = img.copy(QRect(phys_x, phys_y, phys_w, phys_h))
                dest_x = inter.x() - global_rect.x()
                dest_y = inter.y() - global_rect.y()
                painter.drawImage(QRect(dest_x, dest_y, inter.width(), inter.height()), cropped_slice)

            painter.end()
            return canvas.save(dst_path)

    def exec_selection(self, dst_path: str) -> bool:
        for w in self.windows:
            w.show()
            w.raise_()
            w.activateWindow()

        for _ in range(5):
            self.app.processEvents()
            time.sleep(0.02)

        for w in self.windows:
            s_name = w.screen_obj.name()
            title = w.windowTitle()
            cmd = f'[title="{title}"] move window to output "{s_name}", floating enable, fullscreen enable, focus'
            try:
                subprocess.run(["swaymsg", cmd], capture_output=True)
            except Exception:
                pass

        created_app = (QApplication.instance() is self.app)

        if created_app:
            self.app.exec()
        else:
            self.loop = QEventLoop()
            self.loop.exec()

        if self.cancelled or not self.selected_rect:
            return False

        return self.save_cropped_selection(dst_path)


class FreezeSelectionOverlay:
    @classmethod
    def select_area(cls, freeze_files: dict[str, str], dst_path: str) -> bool:
        controller = MultiMonitorSelectionController(freeze_files)
        return controller.exec_selection(dst_path)
