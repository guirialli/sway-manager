import os


class StringUtils:
    @staticmethod
    def truncar_nome_arquivo(nome: str, max_len: int = 20) -> str:
        if not nome or len(nome) <= max_len:
            return nome

        base, ext = os.path.splitext(nome)
        ext_clean = ext.lstrip(".")
        # Reserves space for '...' and extension (e.g. '.png' -> '...png')
        reserved = 3 + (len(ext_clean) if ext_clean else 0)
        tamanho_base = max_len - reserved

        if tamanho_base > 1 and ext_clean:
            return f"{base[:tamanho_base]}...{ext_clean}"
        return f"{nome[:max_len-3]}..."
