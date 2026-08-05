import unittest

from PySide6.QtCore import QPoint, QRect
from PySide6.QtGui import QColor, QImage

from presentation.gui.widgets.freeze_selection_overlay import (
    FrozenScreen,
    SelectionState,
    compose_selection,
)


def solid_image(width: int, height: int, color: str) -> QImage:
    image = QImage(width, height, QImage.Format_ARGB32)
    image.fill(QColor(color))
    return image


class TestSelectionState(unittest.TestCase):
    def test_drag_creates_normalized_rectangle(self):
        state = SelectionState()

        state.begin(QPoint(30, 20))
        state.update(QPoint(10, 5))

        self.assertEqual(state.rectangle, QRect(10, 5, 21, 16))

    def test_finish_rejects_a_selection_smaller_than_minimum(self):
        state = SelectionState()
        state.begin(QPoint(0, 0))

        self.assertIsNone(state.finish(QPoint(4, 4)))

    def test_finish_returns_valid_selection(self):
        state = SelectionState()
        state.begin(QPoint(0, 0))

        self.assertEqual(state.finish(QPoint(5, 5)), QRect(0, 0, 6, 6))


class TestComposeSelection(unittest.TestCase):
    def test_crops_one_screen_at_native_scale(self):
        screen = FrozenScreen(
            "left", QRect(0, 0, 100, 50), solid_image(100, 50, "#ff0000")
        )

        result = compose_selection(QRect(10, 5, 20, 10), [screen])

        self.assertIsNotNone(result)
        self.assertEqual(result.size().width(), 20)
        self.assertEqual(result.size().height(), 10)
        self.assertEqual(result.pixelColor(0, 0), QColor("#ff0000"))

    def test_uses_physical_resolution_for_hidpi_output(self):
        screen = FrozenScreen(
            "hidpi", QRect(0, 0, 100, 50), solid_image(200, 100, "#00ff00")
        )

        result = compose_selection(QRect(10, 5, 20, 10), [screen])

        self.assertIsNotNone(result)
        self.assertEqual(result.size().width(), 40)
        self.assertEqual(result.size().height(), 20)
        self.assertEqual(result.pixelColor(39, 19), QColor("#00ff00"))

    def test_combines_monitors_at_the_highest_participating_scale(self):
        screens = [
            FrozenScreen(
                "left", QRect(0, 0, 100, 50), solid_image(100, 50, "#ff0000")
            ),
            FrozenScreen(
                "right", QRect(100, 0, 100, 50), solid_image(200, 100, "#0000ff")
            ),
        ]

        result = compose_selection(QRect(50, 0, 100, 50), screens)

        self.assertIsNotNone(result)
        self.assertEqual(result.size().width(), 200)
        self.assertEqual(result.size().height(), 100)
        self.assertEqual(result.pixelColor(10, 10), QColor("#ff0000"))
        self.assertEqual(result.pixelColor(150, 10), QColor("#0000ff"))

    def test_preserves_a_transparent_gap_between_outputs(self):
        screens = [
            FrozenScreen(
                "left", QRect(0, 0, 100, 20), solid_image(100, 20, "#ff0000")
            ),
            FrozenScreen(
                "right", QRect(150, 0, 100, 20), solid_image(100, 20, "#0000ff")
            ),
        ]

        result = compose_selection(QRect(50, 0, 150, 10), screens)

        self.assertIsNotNone(result)
        self.assertEqual(result.pixelColor(10, 5), QColor("#ff0000"))
        self.assertEqual(result.pixelColor(75, 5).alpha(), 0)
        self.assertEqual(result.pixelColor(125, 5), QColor("#0000ff"))


if __name__ == "__main__":
    unittest.main()
