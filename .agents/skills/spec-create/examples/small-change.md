# Exemplo: Alteração Pontual com Profundidade Proporcional

Este exemplo demonstra como documentar uma **alteração pontual** sem gerar arquivos artificiais ou redundantes, mantendo a documentação proporcional à complexidade da mudança.

---

## Cenário da Tarefa

- **Domínio afetado**: `src/modules/customer-query/`
- **Alteração**: Adicionar taxa limite (rate limit) de 30 consultas de CNPJ por minuto por conta cliente para evitar bloqueio no gateway da Receita Federal.

---

## Estrutura Resultante

```text
src/
└── modules/
    └── customer-query/
        └── README.md                         ← Atualizado cumulativamente

spec/
└── limite-consultas-cnpj/
    └── behavior.md                           ← Arquivo único coeso
```

---

## 1. Atualização do README do Domínio (`src/modules/customer-query/README.md`)

O README existente foi atualizado para registrar a nova regra de proteção e o link para a spec, mantendo intacta toda a documentação pré-existente:

```markdown
# Domínio: Consulta de Clientes e CNPJ (`customer-query`)

## Responsabilidade
Fornecer serviços de enriquecimento cadastral de pessoas jurídicas consultando serviços públicos externos (Receita Federal e Sintegra).

## Componentes
- `CnpjQueryService`: Orquestra consultas cadastrais com fallback e cache local.
- `ReceitaWsClient`: Adaptador de integração HTTP com o webservice público.
- `QueryRateLimiter`: Interceptor de controle de vazão baseado em Redis.

## Regras Gerais
- Resultados com dados cadastrais válidos são armazenados em cache Redis por 24 horas.
- Consultas a CNPJs inválidos são rejeitadas imediatamente na camada de validação local.
- **Controle de Vazão**: Cada conta cliente possui um teto máximo de 30 consultas de CNPJ por minuto.

## Especificações
- [Controle de Vazão de Consultas CNPJ](../../../spec/limite-consultas-cnpj/behavior.md)
```

---

## 2. Especificação Única: `spec/limite-consultas-cnpj/behavior.md`

Como a alteração não introduziu novas tabelas de banco nem remodelou a arquitetura, todas as informações foram consolidadas de forma coesa em um único arquivo:

```markdown
# Controle de Vazão: Consultas de CNPJ

## Objetivo
Proteger a aplicação contra esgotamento da cota de requisições do provedor externo e impedir abuso de consultas concorrentes por um mesmo cliente.

## Contexto e Problema
A conta corporativa no gateway da Receita Federal impõe penalidade de banimento temporário por IP quando o volume excede 40 requisições simultâneas por minuto. Picos repentinos de um único cliente estavam degradando o serviço para todos os demais.

## Comportamento Efetivamente Implementado
Ao receber uma requisição de consulta de CNPJ, o middleware avalia a taxa de requisições da conta solicitante no Redis utilizando janela deslizante de 60 segundos. Se a contagem ultrapassar 30 requisições na janela corrente, a requisição é rejeitada imediatamente com HTTP `429 Too Many Requests`.

---

## Regras de Negócio

### BR-001 — Limite de Janela Deslizante de CNPJ
- **Condição**: A conta solicitante efetuou 30 consultas nos últimos 60 segundos.
- **Ação**: Bloquear a requisição com código HTTP 429 sem repassar a chamada ao gateway externo.
- **Cabeçalho**: Incluir o cabeçalho `Retry-After: <segundos_restantes>` no retorno HTTP.

### BR-002 — Consultas atendidas por Cache não consom cota
- **Condição**: O CNPJ consultado já está presente no cache local com TTL válido.
- **Ação**: Devolver os dados em cache com status 200 sem incrementar o contador de rate limit da conta.

---

## Contrato de Erro (HTTP 429)

### Headers de Resposta
```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/problem+json
Retry-After: 23
```

### Payload do Erro
```json
{
  "type": "https://api.sga.com.br/errors/rate-limit-exceeded",
  "title": "Limite de consultas excedido",
  "status": 429,
  "detail": "Sua conta atingiu o limite de 30 consultas de CNPJ por minuto. Tente novamente em 23 segundos.",
  "instance": "/api/v1/customers/cnpj/00000000000191"
}
```

---

## Decisões Técnicas

### DEC-001 — Algoritmo de Janela Deslizante em Redis (Sorted Sets)
- **Contexto**: A contagem por janelas fixas (ex: reset no minuto cheio) permitia picos de 60 consultas na borda entre dois minutos.
- **Decisão**: Utilizar Sorted Sets (`ZADD` e `ZREMRANGEBYSCORE`) no Redis para implementar janela deslizante contínua de 60 segundos.
- **Motivação**: Eliminar os picos na borda dos minutos com latência inferior a 2ms por consulta.
```
