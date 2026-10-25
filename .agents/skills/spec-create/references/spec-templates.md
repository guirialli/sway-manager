# Templates de Especificações Técnicas (`spec/<implementacao>/`)

Este documento reúne os modelos estruturais recomendados para cada tipo de especificação lógica. Utilize apenas os arquivos pertinentes à complexidade da sua tarefa.

---

## 1. Ponto de Entrada: `overview.md`

Obrigatório quando a implementação contiver mais de um arquivo em `spec/<implementacao>/`.

```markdown
# <Nome da Funcionalidade / Implementação>

## Objetivo
Descreva em uma frase direta a intenção e finalidade da implementação.

## Contexto
O que motivou esta implementação? Qual era o estado anterior do sistema?

## Problema Resolvido
Qual gargalo, bug, ausência de funcionalidade ou débito técnico foi solucionado.

## Escopo
- Funcionalidades, rotas ou comportamentos cobertos nesta entrega;
- Casos suportados e cenários atendidos.

## Fora de Escopo
- O que deliberadamente não faz parte desta entrega;
- Limitações conhecidas deixadas para iterações futuras.

## Comportamento Geral
Explicação integrada de como a solução atua em regime de produção, articulando seus componentes.

## Arquitetura da Solução
Diagrama textual ou explicação da divisão de responsabilidades entre serviços, adaptadores e storage.

## Impactos
- Mudanças em contratos de API consumidos por outros sistemas;
- Impacto esperado em latência, volume de dados ou carga de banco;
- Efeitos colaterais em domínios correlacionados.

## Compatibilidade
Como a solução lida com clientes em versões anteriores, dados existentes e migrações transparentes.

## Especificações Relacionadas
- [Regras de Negócio](./business-rules.md)
- [Invariantes](./invariants.md)
- [Algoritmo de Resolução](./resolution-flow.md)
- [Contrato de Dados](./payload-contract.md)
- [Persistência](./persistence.md)
- [Decisões Técnicas](./decisions.md)
```

---

## 2. Regras de Negócio: `business-rules.md`

Utilizado sempre que houver regras condicionais, validações de domínio ou políticas de acesso.

```markdown
# Regras de Negócio: <Nome da Implementação>

## BR-001 — <Título Conciso da Regra>
- **Condição**: Quando a condição X ocorrer (ex: `status == PENDING` e `data_vencimento < hoje`).
- **Efeito**: A ação Y deve ser disparada (ex: transição para `OVERDUE` e emissão de alerta).
- **Exceções**: Casos especiais onde a regra não se aplica (ex: contas de teste).

## BR-002 — <Título Conciso da Regra>
- **Condição**: ...
- **Efeito**: ...
- **Exceções**: ...
```

---

## 3. Invariantes do Sistema: `invariants.md`

Utilizado para explicitar regras absolutas e restrições que **jamais** podem ser violadas sob qualquer hipótese.

```markdown
# Invariantes do Sistema: <Nome da Implementação>

## INV-001 — <Título da Invariante>
**Afirmação**: Nenhuma transação financeira pode ser criada com valor menor ou igual a zero.
**Garantia**: Validação em camada de domínio + check constraint no banco de dados (`CHECK (amount > 0)`).
**Recuperação**: Qualquer violação dispara `InvalidDomainStateException` com rollback imediato.

## INV-002 — <Título da Invariante>
**Afirmação**: ...
```

---

## 4. Algoritmos e Fluxos de Decisão: `<nome>-flow.md` / `<nome>-resolution.md`

Utilizado quando há processamento complexo, cálculos, pipelines ou decisões condicionais encadeadas.

```markdown
# Algoritmo de Resolução: <Nome do Processo>

## Etapas do Algoritmo
1. **Entrada e Sanitização**: Recebe os parâmetros de entrada e valida formatos.
2. **Carregamento de Contexto**: Busca entidades de apoio no cache ou banco.
3. **Avaliação Condicional**: Executa a matriz de regras em ordem de precedência.
4. **Transformação**: Mapeia o resultado intermediário no formato final.
5. **Emissão de Eventos / Retorno**: Registra logs de auditoria e devolve a resposta.

## Matriz de Decisão (Tabela Verdade)

| Cenário | Parâmetro A | Parâmetro B | Resultado Esperado | Regra / Invariante |
| :--- | :--- | :--- | :--- | :--- |
| 1 | `true` | `null` | Rejeitado (400 Bad Request) | BR-001 |
| 2 | `true` | `>= 100` | Autorizado com desconto | BR-002 |
| 3 | `false` | Qualquer | Autorizado sem desconto | BR-003 |
```

---

## 5. Contratos de Dados: `payload-contract.md` / `api-contract.md`

Utilizado para registrar alterações em endpoints HTTP, payloads gRPC, mensagens de mensageria ou eventos.

```markdown
# Contrato de Dados: <Nome do Endpoint / Evento>

## Definição
- **Protocolo/Método**: `POST /api/v2/orders`
- **Autenticação**: Bearer Token (JWT com escopo `orders:write`)
- **Content-Type**: `application/json`

## Payload de Entrada (Request Body)
```json
{
  "customer_id": "usr_99812",
  "items": [
    {
      "product_id": "prod_441",
      "quantity": 2,
      "unit_price": 49.90
    }
  ],
  "coupon_code": "PROMO2026"
}
```

## Dicionário de Campos

| Campo | Tipo | Obrigatório? | Semântica e Regras de Validação |
| :--- | :--- | :---: | :--- |
| `customer_id` | String | Sim | Identificador único do cliente. Deve existir na tabela `customers`. |
| `items` | Array | Sim | Lista de itens da compra. Deve conter entre 1 e 50 itens. |
| `items[].quantity` | Integer | Sim | Quantidade do item. Deve ser maior que 0. |
| `coupon_code` | String | Não | Cupom promocional opcional. Case-insensitive. |

## Respostas Possíveis
- `201 Created`: Pedido registrado com sucesso (retorna objeto com `order_id` e `status`).
- `400 Bad Request`: Payload inválido ou violação de constraints.
- `409 Conflict`: Conflito de concorrência ou cupom esgotado.
```

---

## 6. Persistência e Banco de Dados: `persistence.md`

Utilizado sempre que houver modificações de schema, novas tabelas, colunas, chaves estrangeiras ou índices.

```markdown
# Estrutura de Persistência: <Nome da Implementação>

## Tabelas Alteradas / Criadas

### Tabela `orders`
- `discount_amount` (`NUMERIC(10,2)`, default `0.00`): Valor total descontado no pedido.
- `applied_rule_id` (`VARCHAR(64)`, nullable): Identificador da regra de negócio aplicada.

## Constraints e Índices
- `idx_orders_customer_created`: Índice composto em `(customer_id, created_at DESC)` para aceleração de consultas de histórico.
- `chk_orders_discount_non_negative`: `CHECK (discount_amount >= 0)`.

## Semântica de Valores Especiais
- `applied_rule_id = NULL`: Indica pedido sem desconto ou gerado antes da ativação do motor de regras.
- `applied_rule_id = 'MANUAL_OVERRIDE'`: Desconto concedido por intervenção humana do suporte.

## Migrações
- Migration: `migrations/20260917_add_discount_to_orders.sql`
- Estratégia de Rollback: Script idempotente de reversão sem perda de dados históricos de pedidos.
```

---

## 7. Decisões Técnicas e Arquiteturais: `decisions.md`

Utilizado para registrar decisões de design não óbvias com suas motivações e consequências.

```markdown
# Decisões Técnicas: <Nome da Implementação>

## DEC-001 — <Título da Decisão>

### Contexto
Qual era o dilema, limitação técnica ou trade-off enfrentado pela equipe?

### Decisão
Qual caminho técnico foi escolhido e adotado na implementação?

### Motivação
Por que essa opção foi preferida em relação às alternativas avaliadas?

### Alternativas Rejeitadas
- **Alternativa A**: Descrição e motivo pelo qual não foi adotada.
- **Alternativa B**: Descrição e motivo pelo qual não foi adotada.

### Consequências
- **Positivas**: Ganhos de performance, facilidade de manutenção, compatibilidade.
- **Negativas / Débitos**: Maior complexidade pontual, necessidade de refatoração futura.
```
