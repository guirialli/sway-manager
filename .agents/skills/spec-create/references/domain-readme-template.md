# Template: README do Domínio (`<dominio>/README.md`)

Este guia define a estrutura recomendada para o arquivo `README.md` localizado na raiz de cada domínio da aplicação (por exemplo, `src/modules/access-control/README.md` ou `app/billing/README.md`).

---

## 1. Princípios do README de Domínio

1. **Visão Macro e Ponto de Entrada**: Deve permitir a um desenvolvedor novo compreender a responsabilidade do domínio em poucos minutos.
2. **Conceitual e Enxuto**: Não detalhe linhas de código ou métodos específicos; detalhes de implementação pertencem às especificações em `spec/<implementacao>/`.
3. **Manutenção Cumulativa**: Quando uma nova funcionalidade for adicionada ao domínio, atualize este README incorporando os novos conceitos, componentes ou links de especificação **sem apagar a documentação existente válida**.

---

## 2. Template Estrutural

```markdown
# <Nome do Domínio>

## Responsabilidade
Descreva em um ou dois parágrafos a missão e responsabilidade principal deste domínio dentro do sistema.
Exemplo: "Este domínio é responsável pelo gerenciamento de planos, cobranças recorrentes, faturas e controle de inadimplência de assinantes da plataforma."

## Contexto
Onde este domínio se situa na arquitetura geral. Explique que problema de negócio ele resolve e quais outros sistemas/módulos dependem dele ou fornecem dados a ele.

## Conceitos Principais
Defina o vocabulário e termos ubíquos do domínio.
- **Assinatura**: Contrato ativo que vincula um cliente a um plano tarifário.
- **Ciclo de Faturamento**: Período temporal (mensal, anual) no qual as cobranças são apuradas.
- **Fatura**: Registro financeiro gerado para liquidação de um ciclo.

## Componentes
Liste os principais módulos, serviços ou agregados que compõem o domínio:
- `SubscriptionService`: Gestão do ciclo de vida da assinatura (criação, upgrade, cancelamento).
- `InvoiceProcessor`: Apuração e geração de faturas periódicas.
- `PaymentGatewayAdapter`: Abstração de comunicação com operadoras financeiras.

## Regras Gerais
Premissas e diretrizes gerais que governam o domínio:
- Nenhuma fatura pode ser cancelada após confirmação de liquidação financeira.
- Cancelamentos de planos anuais geram estorno proporcional em créditos na plataforma.
- Bloqueio de acesso por inadimplência ocorre automaticamente após 5 dias úteis do vencimento.

## Fluxos Principais
Resumo dos fluxos mais críticos suportados pelo domínio:
1. **Ativação de Assinatura**: Entrada de checkout → Validação de cartão → Criação da conta → Liberação de acesso.
2. **Renovação Automática**: Job noturno → Verificação de vencimento → Cobrança no gateway → Notificação por e-mail.
3. **Tratamento de Inadimplência**: Falha de pagamento → Retentativas automáticas (dias 1, 3 e 5) → Bloqueio da conta.

## Dependências
Bibliotecas, pacotes e serviços internos que este domínio consome:
- `@company/auth-client`: Validação de tokens de usuários.
- `@company/event-bus`: Publicação de eventos de faturamento.
- `Stripe SDK`: Processamento de pagamentos.

## Persistência
Modelos e entidades centrais armazenados pelo domínio:
- Banco de dados principal: PostgreSQL (`billing_db`).
- Tabelas centrais: `subscriptions`, `invoices`, `payment_transactions`, `plans`.
- Estratégia de consistência: Transações ACID para liquidação de faturas.

## Integrações
Sistemas externos ou assíncronos vinculados:
- **Webhooks Stripe**: Recepção assíncrona de confirmações de pagamento via HTTP.
- **Tópico Kafka `billing.events`**: Emissão de eventos `InvoiceGenerated`, `PaymentSucceeded`, `SubscriptionCanceled`.

## Especificações
Links diretos para as especificações técnicas das implementações vinculadas a este domínio:
- [Visão Geral: Cobrança Recorrente](../../spec/cobranca-recorrente/overview.md)
- [Regras de Negócio: Cobrança Recorrente](../../spec/cobranca-recorrente/business-rules.md)
- [Contrato de Webhooks](../../spec/cobranca-recorrente/webhook-contracts.md)
```

---

## 3. Checklist de Validação do README

- [ ] O arquivo está posicionado na pasta raiz do domínio afetado?
- [ ] Os termos usados correspondem à linguagem do negócio e do código?
- [ ] Todos os links da seção `## Especificações` utilizam caminhos relativos válidos para `spec/<implementacao>/`?
- [ ] As informações adicionadas preservaram o conteúdo anterior que ainda é válido?
