# Exemplo: Modo `web`

Cenário: avaliar se `release_version = NULL` em rotina pode ser gravado via migration,
antes de escrever a migration do módulo `dashboard-executivo` (SGA Cloud).

## Perguntas

- P1: A coluna `catalogo_rotinas.release_version` aceita `NULL`?
- P2: Qual o padrão canônico de migration idempotente no TypeORM para esse catálogo?

## Fan-out (2 scouts)

```json
{
  "context": "SGA Cloud, TypeORM, catálogo servicos.catalogo_rotinas. Objetivo: fatos externos + internos antes da migration.",
  "tasks": [
    {
      "name": "WebPadroes",
      "agent": "scout",
      "task": "Pesquise na web o padrão canônico de migrations TypeORM idempotentes e reversíveis (up/down) para colunas versionadas. Retorne 2-3 fontes com URL e o trecho aplicável. Recency: year.",
      "solutionSpace": "resposta factual externa; queries dadas"
    },
    {
      "name": "SchemaLocal",
      "agent": "scout",
      "task": "Em src/migrations/ e no schema, localize a definição de servicos.catalogo_rotinas.release_version (NOT NULL?) e 1 migration de exemplo que altere release_version. Retorne caminho:linha de cada achado.",
      "solutionSpace": "leitura dirigida; diretório conhecido"
    }
  ]
}
```

## Briefing resultante (resumo)

```markdown
## Briefing de Contexto — web — release_version de rotinas

### Perguntas respondidas
- P1: coluna é NOT NULL; NULL é só parâmetro de runtime (-1/admin), nunca persistido [fonte: src/schema/catalogo.sql:42]
- P2: up/down espelhados por chave funcional, sem UUID hardcoded [fonte: https://typeorm.io/migrations]

### Caminhos centrais
- `src/migrations/1801000000000-...ts` — modelo de migration do catálogo (linhas 79-107)

### Lacunas (não comprovadas)
- Semântica de `version == -1` na API — procurar em `src/modules/access-control/` (modo repo)

### Handoff
- Próxima etapa: implementar migration com `module-release-version` usando P1+P2 como restrição
```
