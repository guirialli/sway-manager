from typing import TypeVar

T = TypeVar("T")


class ArrayUtils:
    @classmethod
    def getSafe(cls, lista: list[T], index: int, default: T | None = None) -> T | None:
        if 0 <= index < len(lista) and lista[index] is not None:
            return lista[index]
        return default
