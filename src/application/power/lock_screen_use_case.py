from domain.power.repositories import ILockRepository


class LockScreenUseCase:
    def __init__(self, lock_repo: ILockRepository):
        self.lock_repo = lock_repo

    def execute(self) -> None:
        self.lock_repo.lock_screen()
