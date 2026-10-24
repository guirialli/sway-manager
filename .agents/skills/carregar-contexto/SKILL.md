---
name: carregar-contexto
description: Carrega contexto no agente via scouts paralelos (task/scout): pesquisas web, leitura de specs/docs e varredura de diretórios inteiros. Use ao iniciar tarefa que exige contexto prévio, mapear domínio desconhecido ou reunir evidências antes de implementar, revisar ou documentar.
---

# Skill de Carga de Contexto (`carregar-contexto`)

Esta Skill carrega informação no contexto do agente **antes** da execução principal, usando os
scouts configurados em paralelo. Cobre três fontes: **buscas na web**, **leitura de specs/docs**
e **varredura de diretórios inteiros**. O agente pai planeja, distribui e consolida; os scouts
apenas leem e reportam.

---

## 1. Quando Ativar esta Skill

Ative sempre que a tarefa exigir contexto prévio não presente na conversa:

- Mapear um repositório, domínio ou subsistema desconhecido antes de implementar;
- Reunir specs, ADRs, `README`s, `AGENTS.md` e contratos antes de codificar ou revisar;
- Pesquisar documentação externa, APIs, changelogs ou padrões na web;
- Levantar arquivos, símbolos, fluxos ou testes relacionados a um bug, erro ou refatoração;
- Preparar o terreno para outra skill (`spec-create`, `code-review`, `git-commit-sga`).

NÃO ative para continuações triviais com arquivos já identificados, nem para edições
mecânicas de escopo fechado.

---

## 2. Princípio Fundamental

<critical>
O agente pai NUNCA varre o repositório ou a web ele mesmo quando o volume excede 2-3 leituras. Ele delega a scouts em paralelo via `task` e consolida os achados. Scouts são SOMENTE-LEITURA: nenhum scout edita, commita ou executa mutação.
</critical>

Divisão de papéis:

| Papel | Responsabilidade |
| :--- | :--- |
| Pai (esta sessão) | Classifica o modo, formula o plano de fan-out, dispara os scouts, consolida o briefing |
| Scout (`agent: "scout"`) | Lê arquivos, executa `grep`/`glob`/`read`, pesquisa a web, retorna achados com caminhos e evidências |

Por que fan-out e não leitura sequencial: cada scout tem janela própria de contexto; o pai
recebe apenas resumos com ponteiros (`caminho:linha`, URLs). Volume de leitura não contamina
a sessão principal.

---

## 3. Modos de Carga

| Modo | Fonte | Gatilho | Entrega |
| :--- | :--- | :--- | :--- |
| `web` | `web_search` + `read <URL>` | Docs externas, APIs, padrões, changelogs | Fatos datados com URL-fonte |
| `specs` | `spec/`, `docs/`, `README`, `AGENTS.md`, ADRs | Entender comportamento documentado e regras | Regras/invariantes com caminho-fonte |
| `repo` | Árvore + `grep`/`glob`/`lsp` | Mapear código, fluxos, símbolos, testes | Mapa de arquivos e relações |
| `full` | Combinação dos três acima | Domínio desconhecido, tarefa não trivial | Briefing consolidado multi-fonte |

Regra de seleção: comece pelo modo mais estreito que responde à pergunta. Escalone para
`full` somente quando um modo isolado deixar lacuna concreta. Detalhes e orçamentos por
modo em [`references/modos.md`](references/modos.md).

---

## 4. Workflow Operacional Obrigatório (6 Etapas)

```text
1. Classificar (modo + perguntas a responder)
       ↓
2. Formular alvos (símbolos, paths, queries web — o caminho mais provável primeiro)
       ↓
3. Fan-out (um task batch com context compartilhado + N scouts)
       ↓
4. Monitorar (wait / agent:// — sem re-consulta equivalente)
       ↓
5. Consolidar (briefing com fontes, sem inventar o ausente)
       ↓
6. Handoff (aponta a próxima skill/etapa com o contexto pronto)
```

### Etapa 1 — Classificar

Defina explicitamente: **modo** (`web` | `specs` | `repo` | `full`) e **2–5 perguntas**
que o contexto deve responder (ex: "onde reside a validação de `release_version`?",
"qual o contrato do endpoint X?"). Sem perguntas, não há fan-out — volte ao solicitante.

### Etapa 2 — Formular alvos

Para cada pergunta, registre o alvo mais provável primeiro (arquivo, diretório, símbolo,
query web). Comece pelo caminho conhecido; expanda para repositório inteiro somente se as
etapas anteriores não cobrirem. Consulte `references/omp-tools.md` para a sintaxe exata
de cada ferramenta.

### Etapa 3 — Fan-out

Um único `task` em forma batch: `context` compartilhado + um item por scout. Contrato
obrigatório por item: `agent: "scout"`, `task` auto-suficiente, `solutionSpace` explícito.
No máximo **32 scouts** por chamada; agrupe por afinidade lógica (não por camada física).
Especificação completa em [`references/scout-fanout.md`](references/scout-fanout.md).

```json
{
  "context": "Repo SGA Cloud em /home/user/sga. Convenções: TypeORM, spec/ por contexto lógico.",
  "tasks": [
    {
      "agent": "scout",
      "task": "Mapeie onde release_version é lida em src/modules/access-control/. Retorne arquivos, linhas e condição de cada leitura.",
      "solutionSpace": "causa e localização abertas; 1 subsistema delimitado"
    },
    {
      "agent": "scout",
      "task": "Leia spec/controle-acesso-modulos/ e extraia regras BR-*, INV-* e decisões DEC-* com o arquivo-fonte de cada uma.",
      "solutionSpace": "leitura dirigida; arquivos conhecidos"
    }
  ]
}
```

### Etapa 4 — Monitorar

Resultados entregam automaticamente. Para inspecionar sem consumir: `read proc://<jobId>`.
Para redirecionar um scout em curso: `write agent://<id>` (ex: "ignore `dist/`, foque em
`src/`"). NUNCA repita uma consulta equivalente na mesma tarefa; uma refinada só com
lacuna concreta e novo aspecto (símbolo, erro ou arquivo descoberto).

### Etapa 5 — Consolidar

Monte o briefing no formato da §7 usando SOMENTE o que os scouts comprovaram
(`caminho:linha`, URL, trecho). Lacuna sem evidência é registrada como lacuna —
NUNCA preenchida por inferência. Se um modo inteiro voltar vazio, declare o modo
sem resultado e siga com o disponível.

### Etapa 6 — Handoff

Encerre apontando a próxima etapa com o contexto pronto:

- Implementação → cite os arquivos-alvo e regras aplicáveis;
- Documentação → invoque `spec-create` com o briefing como entrada;
- Revisão → invoque `code-review` com o mapa de impacto parcial já levantado.

---

## 5. Regras de Ouro e Proibições

<critical>
1. Scouts NUNCA escrevem: sem `edit`, `write`, `bash` mutante, commit ou push nos scouts.
2. Orçamento progressivo: ranges e resumos primeiro (`read <arq>:50-100`, diretório página a página); arquivo integral só para referência comprovadamente central.
3. Exclusões padrão em varredura: `.git`, `node_modules`, `vendor`, `.venv`, `__pycache__`, `dist`, `build`, `coverage`, `.next`, `target`, lockfiles e minificados — salvo alvo direto da tarefa.
4. Uma busca inicial + UMA refinada por lacuna concreta. NUNCA varredura precautória ("só para garantir").
5. Memória técnica (`sga-memory-retrieval`) complementa o código atual; NUNCA substitui a validação nos arquivos encontrados.
</critical>

---

## 6. Compatibilidade OMP (Oh My Py)

Recursos do harness usados por esta skill, com evidência na documentação oficial
(`can1357/oh-my-pi`, branch `main`, pasta `docs/`):

- **Skills como capability packs** (`docs/skills.md`): diretório `<root>/<skill>/SKILL.md`,
  frontmatter `name`/`description`, acesso sob demanda via `read skill://<nome>/...`;
  `skill://<nome>` resolve o `SKILL.md`, `skill://<nome>/<path>` resolve dentro da skill.
- **Fan-out de subagentes** (`docs/tools/task.md` + `docs/task-agent-discovery.md`):
  forma batch `{ context, tasks[] }`, agentes bundled `scout`/`reviewer`/`task`/`sonic`,
  descoberta `project .omp/agents` > `user` > extensões > bundled; saída integral em
  `agent://<id>`, transcrição em `history://<id>`, mensagens via `write agent://<id>`,
  concorrência limitada por `task.maxConcurrency` (padrão 32).
- **Leitura universal** (`docs/tools/read.md`): um `path` para arquivos, diretórios,
  arquivos (`:50-100`, `:-N`, `:A-B,C-D`, `:raw`), SQLite (`db:tabela`, `?q=SELECT`),
  PDFs/documentos convertidos, URLs web e esquemas internos (`skill://`, `agent://`,
  `history://`, `memory://root`, `proc://`, `pr://`, `ssh://`).
- **Busca web** (`docs/tools/web_search.md`): `query` com `site:`, `-site:`, `after:`/`before:`,
  `filetype:`, frases entre aspas, `OR`; `recency` dia/semana/mês/ano; cadeia de provedores
  via papel `web`.
- **Code intelligence**: `lsp` (definições, referências, renames), `grep`/`glob` com
  escopo progressivo, `eval` (células py/js persistentes) para agregar e tabular achados.
- **Contexto e memória** (`docs/context-files.md`, `docs/memory.md`): `AGENTS.md`/`CLAUDE.md`
  por profundidade com shadowing por prioridade; memória via backends `local`/`hindsight`/
  `mnemopi`/`sharpshooter` lida por `memory://root` e ferramentas `recall`/`retain`.

Guia de sintaxe e esquemas internos em [`references/omp-tools.md`](references/omp-tools.md).

---

## 7. Formato Obrigatório do Briefing

Toda carga de contexto encerra com este bloco na resposta:

```markdown
## Briefing de Contexto — <modo> — <tópico>

### Perguntas respondidas
- P1: <pergunta> → <resposta em 1-2 linhas> [fonte: <caminho:linha|URL>]

### Caminhos centrais
- `<caminho>` — <papel no fluxo> (linhas <A-B>)

### Relações
- `<origem>` → `<destino>`: <chamada/dependência>

### Regras/fatos apurados
- <fato> [fonte: <caminho:linha|URL>]

### Lacunas (não comprovadas)
- <lacuna> — <onde procurar a seguir>

### Handoff
- Próxima etapa: <implementar|spec-create|code-review|...> com <entradas prontas>
```

Sem fontes rastreáveis, o item pertence a **Lacunas**, nunca a Fatos.

---

## 8. Critérios de Conclusão (Checklist)

- [ ] Modo classificado (`web` | `specs` | `repo` | `full`) e perguntas registradas.
- [ ] Fan-out em batch único com `context` compartilhado; todo item com `solutionSpace`.
- [ ] Nenhum scout executou escrita ou mutação.
- [ ] Exclusões padrão respeitadas; sem abertura integral precautória.
- [ ] Cada fato do briefing tem fonte (`caminho:linha` ou URL); resto está em Lacunas.
- [ ] Handoff aponta a próxima etapa com entradas prontas.

---

## 9. Referências e Exemplos

- 🔀 **Contrato de fan-out (task/scout)**: [`references/scout-fanout.md`](references/scout-fanout.md)
- 🧰 **Ferramentas OMP e esquemas internos**: [`references/omp-tools.md`](references/omp-tools.md)
- 🎛️ **Matriz de modos e orçamentos**: [`references/modos.md`](references/modos.md)
- 🌐 **Exemplo modo `web`**: [`examples/modo-web.md`](examples/modo-web.md)
- 📄 **Exemplo modo `specs`**: [`examples/modo-specs.md`](examples/modo-specs.md)
- 🗂️ **Exemplo modo `repo`**: [`examples/modo-repo.md`](examples/modo-repo.md)
- 🧩 **Exemplo modo `full`**: [`examples/modo-full.md`](examples/modo-full.md)
