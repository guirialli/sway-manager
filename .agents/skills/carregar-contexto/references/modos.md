# Matriz de Modos e Orçamentos

Quando usar cada modo, que perguntas ele responde e onde parar.

## Seleção em 30 segundos

```text
Precisa de fato EXTERNO (docs, API, changelog, padrão)? ── sim ──→ modo web
Precisa do que está DOCUMENTADO (spec, README, ADR)? ─── sim ──→ modo specs
Precisa do que o CÓDIGO faz (símbolos, fluxos, testes)?  sim ──→ modo repo
Mais de um acima, ou domínio desconhecido? ───────────── sim ──→ modo full
Tudo já está na conversa? ────────────────────────────── sim ──→ SEM skill (trivial)
```

Escalone estreito → amplo só com lacuna concreta. `full` = `web` + `specs` + `repo`
num único batch, um grupo de scouts por fonte.

## Modo `web` — fatos externos

- **Responde**: como a API/lib se comporta, o que mudou no changelog, qual o padrão canônico.
- **Orçamento**: 1 `web_search` inicial + no máximo 1 refinada por lacuna; `read <URL>`
  só nas 2–3 fontes realmente aplicáveis (resumo da busca decide, não precaução).
- **Exigências**: toda afirmação datada com URL-fonte; `recency` explícito quando o tema
  é versionado; contradiz local × web → vale o local, registre o conflito.
- **Pare quando**: 2 fontes independentes convergirem, ou a cadeia de provedores falhar
  (declare modo sem resultado).
- **Exemplo**: [`examples/modo-web.md`](../examples/modo-web.md).

## Modo `specs` — comportamento documentado

- **Responde**: regras `BR-*`, invariantes `INV-*`, decisões `DEC-*`, contratos, fluxos descritos.
- **Orçamento**: `spec/<impl>/` + `README` do domínio + `AGENTS.md` aplicável primeiro;
  `docs/`, ADRs e `memory://root` só com lacuna. Arquivo integral só se central e citado
  por outros.
- **Exigências**: cada regra com identificador estável + arquivo-fonte; reflita o
  documentado (divergência doc × código vai para Lacunas, não é "corrigida" aqui);
  links relativos entre specs preservados na citação.
- **Pare quando**: perguntas cobertas pelos docs lidos, ou docs inexistentes
  (declare e siga para `repo` se autorizado).
- **Exemplo**: [`examples/modo-specs.md`](../examples/modo-specs.md).

## Modo `repo` — o que o código faz

- **Responde**: onde mora cada comportamento, quem chama quem, fluxos ponta a ponta,
  testes existentes.
- **Orçamento**: escopo progressivo obrigatório —
  1. caminho/símbolo exato → 2. diretório + extensões → 3. repo inteiro.
  Ranges e resumos primeiro; integral só no arquivo comprovadamente central.
  Mapa do projeto (`project_map_query` / `sga-memory map query`, 350 tokens) antes
  de varrer repo desconhecido, quando disponível.
- **Exclusões padrão**: `.git`, `node_modules`, `vendor`, `.venv`, `__pycache__`,
  `dist`, `build`, `coverage`, `.next`, `.nuxt`, `target`, lockfiles, minificados.
- **Exigências**: mapa com `caminho:linha` por afirmação; distinção código atual ×
  memória técnica (`sga-memory-retrieval` complementa, NUNCA substitui; cite a nota-fonte).
- **Pare quando**: definição + chamadores + testes do alvo localizados, ou 1 refinada
  sem progresso (declare a lacuna).
- **Exemplo**: [`examples/modo-repo.md`](../examples/modo-repo.md).

## Modo `full` — domínio desconhecido

- **Responde**: o conjunto das perguntas dos três modos, consolidadas sem duplicação.
- **Orçamento**: um batch, grupos por fonte (ex: 2 scouts `repo` + 1 `specs` + 1 `web`
  = 4 scouts). Teto: 32 scouts por chamada; agrupe por afinidade lógica.
  Uma refinada por grupo com lacuna, nunca varredura precautória global.
- **Exigências**: briefing único (§7 do SKILL.md); cada fato marcado com a fonte
  (`[código]`, `[doc]`, `[web]`); conflito entre fontes registrado, não resolvido
  por inferência.
- **Pare quando**: cada pergunta tem resposta com fonte ou lacuna declarada.
- **Exemplo**: [`examples/modo-full.md`](../examples/modo-full.md).

## Tabela de orçamentos

| Recurso | Teto por modo | Teto no `full` (por grupo) |
| :--- | :--- | :--- |
| Scouts por batch | 32 | 32 (total) |
| Buscas web iniciais | 1 por pergunta | 1 por pergunta web |
| Refinadas | 1 por lacuna concreta | 1 por lacuna, por grupo |
| Leituras integrais | só central comprovado | só central comprovado |
| Notas de memória abertas | ≤2 integrais | ≤2 integrais |
