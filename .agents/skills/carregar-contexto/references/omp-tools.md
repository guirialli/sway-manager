# Ferramentas OMP para Carga de Contexto

Sintaxe operacional das ferramentas que os scouts usam. Fonte: `docs/tools/*.md`
do `can1357/oh-my-pi`. O pai planeja; o scout executa.

## `read` — leitura universal

Um `path` resolve arquivos, diretórios, SQLite, PDFs, URLs e esquemas internos.

```text
read <arquivo>              # resumo estruturado se >100 linhas (elide com "…")
read <arquivo>:50-100       # range exato (1 linha antes + 3 depois no non-raw)
read <arquivo>:-60          # últimas 60 linhas
read <arquivo>:5-16,960-973 # múltiplos ranges
read <arquivo>:raw          # verbatim, sem resumo nem hashline
read <diretório>            # árvore (profundidade 2, 12 entradas/dir, recentes primeiro)
read <arquivo>:conflicts    # índice de conflitos de merge (registra conflict://)
```

- Arquivo grande: leia ranges sob demanda a partir do resumo (`[…NNln elided; re-read …]`);
  NUNCA abra o integral por precaução.
- SQLite: `db.sqlite` (lista tabelas), `db.sqlite:tabela` (schema + 5 amostras),
  `db.sqlite:tabela?where=…&limit=…` (consulta; `where` sem `;`/`UNION`/`LIMIT`),
  `db.sqlite?q=SELECT …` (SQL cru).
- Documentos (`pdf/docx/xlsx/pptx`): convertidos para texto; ranges aplicam-se ao convertido.
- Imagens: `read img.png` (inline) ou `read img.png?q=<pergunta>` (resposta do modelo de visão).

## Esquemas internos (todos via `read`, salvo nota)

| Esquema | Uso nesta skill |
| :--- | :--- |
| `skill://<nome>/...` | Ler outra skill: `skill://spec-create/SKILL.md`, `skill://code-review/references/...` |
| `agent://<id>` | Saída integral do scout; `/chave/índice` extrai campo JSON |
| `history://<id>` | Transcrição do scout; `history://` bare lista registrados |
| `memory://root` | Resumo injetado no startup; `memory://root/MEMORY.md`, `memory://root/skills/<n>/SKILL.md` |
| `proc://<id>` | Status + saída parcial de job/scout sem consumir a entrega |
| `pr://<N>` / `issue://<N>` | PRs/issues como filesystem (`/diff`, `/diff/<i>`, `?comments=0`) |
| `ssh://host/<path>` | Arquivo/dir remoto UTF-8 (≤1 MiB); `:` `?` `#` como `%3A` `%3F` `%23` |
| `local://<arq>.md` | Raiz compartilhada pai↔scouts (fundo comum sem `task.batch`) |
| `xd://<tool>` | Documentação de device montado (`read xd://` lista) |

Escritas: `write agent://<id>` mensageia scout vivo; `write proc://<id>/kill` cancela job.

## `grep` / `glob` — busca com escopo progressivo

```text
grep "release_version" path=src/modules/access-control  # literal primeiro
glob src/**/*.ts                                        # descoberta por nome
```

Ordem obrigatória: caminho/símbolo conhecido → diretório + extensões prováveis →
repositório inteiro. Expanda literal → conceito/alias/chamada só quando necessário.
Contexto ao redor do match, não abertura de todos os resultados.

## `web_search` + `read <URL>`

```text
web_search "TypeORM migration idempotente site:typeorm.io"
web_search "changelog <lib> after:2026-01-01"
read https://docs.exemplo.com/api          # renderizado como markdown com âncoras
read https://arxiv.org/pdf/xxxx.pdf        # PDF convertido
```

- `query`: `site:`/`-site:`, `after:`/`before:`, `inurl:`, `intitle:`, `filetype:`,
  `"frase exata"`, `-termo`, `OR`. `recency`: `day|week|month|year`.
- O `read` de URL pagina do cache (`:50-100` não refetch). `read` cita a fonte
  (`URL:`, `Content-Type:`, `Method:`).
- Cadeia de provedores via papel `web`; falha total retorna `Error: ...` como texto —
  trate como modo sem resultado, não como exceção fatal.

## `lsp` — inteligência de código

Definições, referências, hover, renames via workspace (`willRenameFiles` atualiza
re-exportações). Use para: quem chama X, onde X é definido, hierarquia de tipos.
Via device montado: `write xd://lsp` (requer `tools.xdev`).

## `eval` — agregação de achados

Células persistentes `py`/`js` para tabular o que os scouts retornaram
(contar ocorrências, montar matriz de decisão, gerar gráfico). Ponte tool↔kernel:
`read`/`write`/`tool.*` dentro da célula; `agent()`/`workpool()` para fan-out
programático; NUNCA `python -c`/`bun -e` via `bash` para ad-hoc.

## `task` / `wait` / `todo` / `ask`

- `task`: o fan-out (ver `scout-fanout.md`). `wait`: bloqueia até o próximo resultado
  quando nada mais há a fazer — NUNCA polling.
- `todo`: o pai registra as perguntas como checklist; o scout NÃO planeja pelo pai.
- `ask`: trave o modo com o usuário só diante de trade-off real (2–5 opções);
  padrão = modo mais estreito viável, sem perguntar.
