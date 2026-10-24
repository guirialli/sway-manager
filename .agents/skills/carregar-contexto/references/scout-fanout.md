# Contrato de Fan-out (task + scout)

Como distribuir leitura em scouts paralelos no OMP. Baseado em `docs/tools/task.md`
e `docs/task-agent-discovery.md` do `can1357/oh-my-pi`.

## Forma batch (única forma nesta skill)

```json
{
  "context": "<fundo compartilhado: repo, convenções, objetivo da carga>",
  "tasks": [
    {
      "name": "DocsCatalogo",
      "agent": "scout",
      "task": "<instruções auto-suficientes>",
      "solutionSpace": "<o que está aberto vs dado>"
    }
  ]
}
```

- `context` (obrigatório): fundo comum injetado no system prompt de TODOS os scouts
  (path do repo, stack, objetivo, exclusões). NUNCA repita o contexto inteiro em cada item.
- `tasks[]`: um item por scout. Campos obrigatórios por item: `agent: "scout"`,
  `task` completo, `solutionSpace` explícito. `name` CamelCase ≤32 (opcional, recomendado).
- NUNCA top-level `agent` em chamada batch — o agente vai em cada item.
- `solutionSpace` descreve abertura do problema, não volume. Exemplos:
  - `"leitura dirigida; arquivos conhecidos"` (specs listadas);
  - `"causa e localização abertas; 1 subsistema delimitado"` (repo desconhecido);
  - `"resposta factual externa; queries dadas"` (web).
- Sem `task.batch`: um `task` por chamada, fundo via arquivo `local://`.

## O agente `scout`

- Agente bundled somente-leitura: `read` sem resumos estruturais (`read-summarize: false`,
  texto verbatim), sem pinagem de skill por chamada. Ideal para varredura e extração.
- Alternativas e quando preferir:
  - `reviewer`: só se o item já for julgamento de qualidade (não é carga de contexto);
  - `sonic`: varredura mecânica massiva sem raciocínio (listar arquivos por padrão);
  - `task` (genérico): quando o item exigir síntese intermediária complexa.
- Precedência de descoberta (primeiro vence, nome exato, case-sensitive):
  projeto `.omp/agents` > usuário `~/.omp/agent/agents` > pacotes de extensão >
  plugins marketplace > bundled. Um `scout` customizado no projeto sombreia o bundled.

## Limites rígidos

- Concorrência: `task.maxConcurrency` (padrão **32**). NUNCA mais de 32 scouts por chamada;
  agrupe por afinidade lógica quando estourar.
- Recursão: `task.maxRecursionDepth` (padrão 2). Scouts NÃO disparam sub-scouts;
  aprofunde via `write agent://<id>` no scout existente.
- Orçamento de saída por scout: ~500.000 bytes / 5000 linhas truncam no resumo inline,
  mas o integral fica em `agent://<id>`. Peça ao scout para responder enxuto com
  ponteiros (`caminho:linha`) em vez de colar arquivos inteiros.
- Sem `effort` por item salvo com `task.enableEffort=true`; sem `isolated:true` —
  scouts compartilham o checkout do pai (leitura não precisa de isolamento).

## Ciclo de vida e coordenação

- Resultados entregam automaticamente como async-result. Entrega consumida uma vez;
  o integral permanece em `agent://<id>`.
- Inspecionar sem consumir: `read proc://<jobId>` (status + saída parcial).
- Redirecionar scout vivo: `write agent://<id>` com a nova instrução
  (ex: "ignore `dist/`, foque em `src/`"). Vale para follow-up: prefira mensagear
  o scout existente a disparar um novo — ele já tem o contexto.
- Transcrição do que um scout fez: `history://<id>`. Descobrir scouts registrados:
  `history://` bare.
- `write agent://all` é broadcast — NUNCA use para instrução específica de um scout.

## Anti-padrões (NUNCA)

- NUNCA `agent: "scout"` sem `solutionSpace` — o classificador `auto` usa esse campo.
- NUNCA fatiar por camada física (`models`, `controllers`, `views` em scouts separados).
  Fatie por pergunta/afinidade lógica (fluxo, regra, contrato).
- NUNCA pedir escrita, execução mutante ou commit ao scout. Se precisar de prova
  executável, o PAI executa após consolidar.
- NUNCA repetir consulta equivalente; refinada só com lacuna concreta + aspecto novo.
- NUNCA `task` com `task` vago ("pesquise sobre X"). Task boa = escopo + verbo +
  formato de retorno + exclusões.
