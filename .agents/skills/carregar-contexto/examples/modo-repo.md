# Exemplo: Modo `repo`

Cenário: bug — rotina de módulo inativo aparece para cliente normal. Mapear o fluxo
de resolução antes de corrigir.

## Perguntas

- P1: Onde `release_version` e `is_active` são avaliados no fluxo de listagem?
- P2: Quais testes cobrem esse fluxo?

## Fan-out (2 scouts por afinidade lógica)

```json
{
  "context": "Repo SGA Cloud em ./ . Exclusões: dist/, coverage/, node_modules/. Objetivo: mapa do fluxo de resolução de rotinas.",
  "tasks": [
    {
      "name": "FluxoResolucao",
      "agent": "scout",
      "task": "Partindo de src/modules/access-control/, mapeie o fluxo de listagem de rotinas: entrypoint, services, queries que filtram por is_active/release_version. Escopo progressivo (diretório primeiro, repo só se faltar). Retorne caminho:linha por etapa.",
      "solutionSpace": "causa e localização abertas; 1 subsistema delimitado"
    },
    {
      "name": "TestesFluxo",
      "agent": "scout",
      "task": "Localize testes (unitários/integração) que cubram listagem de rotinas por versão ou módulo inativo em test/ ou *.spec.ts. Retorne arquivo + caso coberto, sem executar a suíte.",
      "solutionSpace": "localização aberta; padrão de nome conhecido"
    }
  ]
}
```

## Briefing resultante (resumo)

```markdown
## Briefing de Contexto — repo — listagem de rotinas

### Caminhos centrais
- `src/modules/access-control/services/routine-resolver.ts` — avalia is_active + release_version (linhas 41-78)
- `test/access-control/routine-resolver.spec.ts` — cobre módulo ativo/inativo, não cobre versão -1

### Relações
- `RoutineController.list` → `RoutineResolver.resolve` → `catalogo_rotinas` (via TypeORM query builder)

### Lacunas (não comprovadas)
- Tratamento de `version == -1`: nenhum teste localizado — verificar handlers

### Handoff
- Próxima etapa: corrigir com o fluxo mapeado; após o fix, code-review com este mapa como impacto parcial
```
