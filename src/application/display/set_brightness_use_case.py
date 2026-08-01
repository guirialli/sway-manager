from domain.display.repositories import IBrightnessRepository


class SetBrightnessUseCase:
    def __init__(self, repository: IBrightnessRepository):
        self.repository = repository

    def execute_change(self, direction: str, step: int = 5) -> int:
        cur = self.repository.get_current_percentage()
        if direction == "up":
            target = cur + step
        elif direction == "down":
            target = cur - step
        else:
            target = cur

        return self.repository.set_brightness(target)

    def get_current(self) -> int:
        return self.repository.get_current_percentage()
