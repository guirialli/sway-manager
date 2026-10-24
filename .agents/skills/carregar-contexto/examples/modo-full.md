# Exemplo: Modo `full`

Cenário: onboarding em domínio desconhecido — assumir o módulo `customer-query`
(rate limit de CNPJ) sem nenhum contexto prévio.

## Perguntas

- P1: O que o domínio faz e quais regras o governam? (specs)
- P2: Onde mora a lógica de vazão no código? (repo)
- P3: Qual o padrão externo de rate limit usado como referência? (web)

## Fan-out (3 scouts, um grupo por fonte)

```json
{
  "context": "Módulo src/modules/customer-query/, specs em spec/limite-consultas-cnpj/. Objetivo: briefing multi-fonte para onboarding.",
  "tasks": [
    {
      "name": "DocsDominio",
      "agent": "scout",
      "task": "Leia src/modules/customer-query/README.md e spec/limite-consultas-cnpj/behavior.md. Extraia responsabilidade, conceitos, regras e fluxos com arquivo-fonte.",
      "solutionSpace": "leitura dirigida; arquivos conhecidos"
    },
    {
      "name": "CodigoVazao",
      "agent": "scout",
      "task": "Mapeie em src/modules/customer-query/ onde o teto de 30 consultas/min é aplicado (interceptor, Redis, configuração). Retorne caminho:linha por etapa do fluxo.",
      "solutionSpace": "localização aberta; 1 subsistema delimitado"
    },
    {
      "name": "PadraoExterno",
      "agent": "scout",
      "task": "Pesquise na web padrões de rate limiting por conta (token bucket vs fixed window) com 2 fontes e URLs. Não traga código pronto, só o comparativo aplicável.",
      "solutionSpace": "resposta factual externa; queries dadas"
    }
  ]
}
```

## Briefing resultante (resumo)

```markdown
## Briefing de Contexto — full — customer-query

### Perguntas respondidas
- P1 [doc]: teto de 30 consultas CNPJ/min por conta; cache Redis 24h [fonte: spec/.../behavior.md]
- P2 [código]: `QueryRateLimiter` aplica o teto antes do `ReceitaWsClient` [fonte: src/.../rate-limiter.ts:17]
- P3 [web]: fixed window basta para teto simples; token bucket só se precisar burst [fonte: https://...]

### Lacunas (não comprovadas)
- Estratégia de fallback quando Redis indisponível — próxima varredura (modo repo)

### Handoff
- Próxima etapa: implementar fallback com P1+P2 como restrição; documentar via spec-create
```
