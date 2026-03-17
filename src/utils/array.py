from typing import TypeVar

T = TypeVar("T")


class ArrayUtils:
    @classmethod
    def getSafe[T](cls, lista: list[T], index: int) -> T | None:
        return lista[index] if len(lista) >= index + 1 else None
