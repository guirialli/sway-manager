---
name: spec-create
description: Skill permanente para documentação técnica orientada à implementação. Garante que toda feature, alteração funcional, refatoração relevante ou mudança de regra de negócio produza e mantenha atualizados o README.md do domínio e especificações modulares em spec/<implementacao>/ baseadas no comportamento efetivo.
---

# Skill de Documentação Orientada à Implementação (`spec-create`)

Esta Skill atua como uma **regra permanente e obrigatória durante tarefas de desenvolvimento de software**. Sua responsabilidade é garantir que toda implementação, alteração funcional, nova feature, refatoração relevante ou mudança de regra de negócio produza e mantenha documentação técnica estruturada junto ao código.

---

## 1. Princípio Fundamental

A documentação gerada por esta Skill deve responder com precisão à seguinte pergunta:

> **"Como essa funcionalidade funciona, quais regras governam seu comportamento e quais condições precisam continuar verdadeiras em futuras alterações?"**

### O que NÃO fazer
A documentação técnica **NÃO deve ser um mero inventário de código**:
- ❌ Não listar simplesmente arquivos, classes, interfaces, métodos ou funções;
- ❌ Não parafrasear o código com descrições óbvias (ex: "O service chama o repository");
- ❌ Não descrever apenas o que foi solicitado inicialmente na tarefa (requisito/intenção), ignorando o que foi efetivamente construído.

### O que documentar prioritariamente
A documentação deve concentrar-se na essência comportamental e arquitetural da solução:
- ✅ **Contexto e Problema**: por que a alteração existe e qual problema ela resolve;
- ✅ **Comportamento Efetivamente Implementado**: como o sistema reage em cada cenário;
- ✅ **Regras de Negócio Explícitas**: com identificadores estáveis (`BR-001`, `BR-002`);
- ✅ **Invariantes do Sistema**: condições que nunca podem ser violadas (`INV-001`);
- ✅ **Decisões Técnicas e Arquiteturais**: motivação, alternativas e trade-offs (`DEC-001`);
- ✅ **Contratos de Dados e APIs**: formato, campos, semântica e exemplos reais;
- ✅ **Algoritmos e Fluxos de Decisão**: etapas lógicas e tabelas verdade de processamento;
- ✅ **Estados e Exceções**: transições de estado, erros tratados e edge cases;
- ✅ **Persistência e Migrações**: tabelas, colunas, constraints, dados sentinela e compatibilidade;
- ✅ **Integrações e Dependências**: interações com outros serviços, filas e bibliotecas;
- ✅ **Compatibilidade e Impactos**: impacto em consumidores existentes e estratégias de transição.

---

## 2. Estrutura Arquitetural de Documentação

A documentação divide-se em duas camadas complementares e simultâneas:

```text
raiz-do-projeto/
├── <dominio>/
│   ├── README.md               ← Camada 1: Visão macro do domínio afetado
│   └── ... (código do domínio)
│
└── spec/
    └── <implementacao>/        ← Camada 2: Detalhamento profundo da implementação
        ├── overview.md         ← Ponto de entrada da implementação
        ├── business-rules.md   ← Regras de negócio identificadas
        ├── ...                 ← Specs por contexto lógico (fluxos, contratos, persistência)
        └── decisions.md        ← Decisões técnicas não óbvias
```

### Exemplo Prático de Estrutura

Para uma implementação de controle de acesso a módulos (`controle-acesso-modulos`) dentro do domínio `src/modules/access-control/`:

```text
src/
└── modules/
    └── access-control/
        ├── README.md                           ← Documentação macro do domínio
        ├── controllers/
        ├── services/
        └── repositories/

spec/
└── controle-acesso-modulos/
    ├── overview.md                             ← Visão geral da implementação
    ├── business-rules.md                       ← Regras de negócio (BR-001, BR-002...)
    ├── invariants.md                           ← Invariantes do sistema (INV-001...)
    ├── module-resolution.md                    ← Algoritmo e fluxo de resolução
    ├── version-resolution.md                   ← Lógica condicional de versões
    ├── payload-contract.md                     ← Contrato de entrada e saída
    ├── persistence.md                          ← Estrutura do banco, constraints e migrations
    ├── compatibility.md                        ← Compatibilidade com clientes antigos
    └── decisions.md                            ← Decisões arquiteturais (DEC-001...)
```

---

## 3. Regra de Organização das Specs: Contexto Lógico

<critical>
Cada arquivo de especificação DEVE representar um CONTEXTO LÓGICO COESO, e NÃO uma camada física do código.
</critical>

- **Certo (orientado ao domínio e comportamento)**:
  - `spec/<implementacao>/business-rules.md`
  - `spec/<implementacao>/module-resolution.md`
  - `spec/<implementacao>/version-resolution.md`
  - `spec/<implementacao>/payload-contract.md`
  - `spec/<implementacao>/persistence.md`
  - `spec/<implementacao>/decisions.md`

- **Errado (orientado à organização física do código)**:
  - ❌ `spec/<implementacao>/service-a.md`
  - ❌ `spec/<implementacao>/controller-b.md`
  - ❌ `spec/<implementacao>/repository-c.md`
  - ❌ `spec/<implementacao>/database-sql.md`

A especificação deve refletir **o domínio, o comportamento e as regras**, permitindo compreender o funcionamento mesmo se o código for refatorado ou as classes renomeadas.

---

## 4. Camada 1: README do Domínio (`<dominio>/README.md`)

Toda pasta raiz de domínio afetada por uma implementação **DEVE possuir ou atualizar** o arquivo:

```text
<dominio>/README.md
```

### Objetivo do README de Domínio
Permitir que qualquer desenvolvedor entenda rapidamente:
1. Qual é a responsabilidade do domínio;
2. Qual problema ele resolve;
3. Quais são os principais conceitos e regras gerais;
4. Quais são seus componentes, fluxos, persistência e integrações;
5. Quais especificações detalhadas existem para as implementações do domínio.

### Regra de Manutenção do README
- Se o arquivo já existir: **atualizá-lo de forma cumulativa**, preservando toda documentação válida existente e incorporando os novos conceitos, fluxos ou referências.
- Se não existir: **criá-lo** seguindo a estrutura padronizada.
- O README deve permanecer **enxuto e conceitual**. Detalhes profundos da mecânica interna pertencem a `spec/<implementacao>/`.

### Estrutura Padrão do `<dominio>/README.md`

```markdown
# <Nome do Domínio>

## Responsabilidade
Breve resumo da responsabilidade primária deste domínio no sistema.

## Contexto
Onde este domínio se encaixa na arquitetura global e que problema ele resolve.

## Conceitos Principais
Definição dos termos de negócio fundamentais usados neste domínio.

## Componentes
Visão geral dos principais módulos, serviços ou blocos que compõem o domínio.

## Regras Gerais
Diretrizes e premissas gerais que governam o domínio.

## Fluxos Principais
Fluxos de ponta a ponta mais comuns executados pelo domínio.

## Dependências
Módulos internos e bibliotecas externas necessárias para a operação deste domínio.

## Persistência
Modelos de dados principais, tabelas centrais e armazenamento de estado.

## Integrações
Sistemas externos, mensageria, eventos ou APIs que se conectam a este domínio.

## Especificações
Links relativos para as especificações técnicas das implementações vinculadas a este domínio:
- [Visão geral: Controle de Acesso](../../spec/controle-acesso-modulos/overview.md)
- [Regras de negócio: Controle de Acesso](../../spec/controle-acesso-modulos/business-rules.md)
```

Consulte [`references/domain-readme-template.md`](references/domain-readme-template.md) para o template detalhado.

---

## 5. Camada 2: Especificações Técnicas (`spec/<implementacao>/`)

### 5.1. Ponto de Entrada: `overview.md`
Sempre que a implementação contiver mais de um arquivo de especificação, criar obrigatoriamente:

```text
spec/<implementacao>/overview.md
```

Estrutura recomendada:
```markdown
# <Nome da Implementação>

## Objetivo
Objetivo direto e claro da alteração.

## Contexto
Cenário atual e motivação que levaram a esta implementação.

## Problema Resolvido
Problema concreto resolvido pela entrega.

## Escopo
O que foi implementado e coberto pela tarefa.

## Fora de Escopo
O que explicitamente não foi tratado ou ficou para etapas futuras.

## Comportamento Geral
Visão integrada de como a funcionalidade opera no dia a dia.

## Arquitetura da Solução
Como as peças foram articuladas e integradas.

## Impactos
Impactos colaterais em outros domínios, desempenho ou infraestrutura.

## Compatibilidade
Garantias de retrocompatibilidade com versões ou clientes anteriores.

## Especificações Relacionadas
- [Regras de negócio](./business-rules.md)
- [Resolução de módulos](./module-resolution.md)
- [Contrato de retorno](./payload-contract.md)
- [Persistência](./persistence.md)
- [Decisões técnicas](./decisions.md)
```

---

### 5.2. Regras de Negócio: `business-rules.md`
Quando a implementação introduzir ou alterar regras de negócio, criar:

```text
spec/<implementacao>/business-rules.md
```

Toda regra deve possuir um **identificador estável** (`BR-001`, `BR-002`, etc.) e descrição clara da condição e do efeito:

```markdown
# Regras de Negócio

## BR-001 — Recurso ainda não lançado
Quando `release_version` for `NULL`, o recurso é considerado ainda não lançado para clientes comuns.

## BR-002 — Versão administrativa
Quando a versão recebida na requisição for `-1`, funcionalidades com `release_version = NULL` podem ser disponibilizadas, desde que o módulo correspondente esteja com `is_active = true`.

## BR-003 — Dependência de ativação do módulo
Uma rotina nunca pode ser disponibilizada quando seu módulo pai estiver inativo, independentemente da versão do cliente.
```

Esses identificadores devem ser referenciados posteriormente no código, testes, commits e PRs.

---

### 5.3. Invariantes do Sistema: `invariants.md`
Quando houver condições que obrigatoriamente devem permanecer verdadeiras em qualquer circunstância (invioláveis):

```text
spec/<implementacao>/invariants.md
```

Utilizar identificadores `INV-001`, `INV-002`:

```markdown
# Invariantes do Sistema

## INV-001 — Versão obrigatória em rotinas publicadas
Toda rotina ativa com status publicado deve obrigatoriamente possuir uma versão associada.

## INV-002 — Isolamento de módulo inativo
Nenhuma rotina de um módulo inativo pode ser executada ou listada, sob qualquer parâmetro de consulta.

## INV-003 — Limite estrito de versão do cliente
Uma versão normal do cliente nunca pode acessar uma rotina cuja versão mínima seja superior à versão atual do cliente.
```

---

### 5.4. Fluxos e Algoritmos de Decisão: `<nome>-flow.md` ou `<nome>-resolution.md`
Quando houver lógica de decisão não trivial, criar uma spec dedicada:
- Descrever o algoritmo conceitual numerado;
- Utilizar **tabelas verdade / matrizes de decisão** para condições compostas:

```markdown
# Resolução de Disponibilidade de Rotinas

## Fluxo Conceitual
1. Identificar os módulos contratados pelo cliente.
2. Filtrar módulos com status ativo.
3. Obter as rotinas associadas aos módulos ativos.
4. Avaliar `release_version` de cada rotina contra a versão do cliente.
5. Aplicar exceções administrativas (`version == -1`).
6. Montar o payload final de rotinas autorizadas.

## Matriz de Decisão

| Módulo Ativo | release_version | Versão Cliente | Rotina Disponível? | Regra Aplicada |
| :--- | :--- | :--- | :--- | :--- |
| Não | Qualquer | Qualquer | **Não** | BR-003 |
| Sim | `NULL` | Normal | **Não** | BR-001 |
| Sim | `NULL` | `-1` (Admin) | **Sim** | BR-002 |
| Sim | `<= Versão Cliente` | Normal | **Sim** | Fluxo padrão |
| Sim | `> Versão Cliente` | Normal | **Não** | INV-003 |
```

---

### 5.5. Contratos de Dados: `payload-contract.md` ou `api-contract.md`
Sempre que a implementação alterar contratos de APIs, payloads de eventos ou interfaces:
- Documentar campos, tipos, obrigatoriedade e exemplos reais;
- Documentar a **semântica dos dados** (o que cada valor significa), e não apenas o formato JSON;
- Documentar o impacto em consumidores legados:

```markdown
# Contrato de Retorno: Consulta de Rotinas

## Endpoint
`GET /api/v1/access-control/modules`

## Estrutura do Payload (Response 200)
```json
{
  "modules": [
    {
      "id": "mod-financial",
      "name": "Módulo Financeiro",
      "routines": [
        {
          "code": "FIN-001",
          "name": "Conciliação Bancária",
          "min_version": "2026.1"
        }
      ]
    }
  ]
}
```

## Dicionário de Campos e Semântica

| Campo | Tipo | Obrigatório | Descrição e Semântica |
| :--- | :--- | :--- | :--- |
| `modules` | Array | Sim | Lista de módulos ativos e disponíveis para a conta. |
| `modules[].routines` | Array | Sim | Rotinas filtradas e autorizadas conforme versão informada. |
| `modules[].routines[].min_version` | String/Null | Não | Versão mínima do cliente para execução. `null` indica recurso restrito a ambiente admin. |
```

---

### 5.6. Persistência e Banco de Dados: `persistence.md`
Quando houver alteração de banco de dados, migrations, triggers ou constraints:

```markdown
# Estrutura de Persistência e Migrações

## Tabelas Afetadas

### Tabela `access_routines`
- `release_version` (VARCHAR, NULL): Versão de lançamento.
- `module_id` (INT, NOT NULL, FK -> `access_modules.id`): Módulo vinculado.

## Semântica de Valores Especiais
- `release_version = NULL`: Recurso em desenvolvimento ou homologação interna; não visível para clientes em produção regular.
- `-1`: **Atenção:** Valor especial reservado para chamadas de API administrativa. **NUNCA** deve ser persistido nesta coluna.

## Migrações
- Arquivo de migração: `migrations/20260917_add_release_version_to_routines.sql`
- Rollback: `ALTER TABLE access_routines DROP COLUMN release_version;`
- Seeds necessários: Rotinas do módulo fiscal populadas com versão base `2026.1`.
```

---

### 5.7. Decisões Técnicas e Arquiteturais: `decisions.md`
Registrar decisões técnicas não óbvias que poderiam ser questionadas ou revertidas sem o conhecimento do contexto original:

```markdown
# Decisões Técnicas

## DEC-001 — Manter formato planificado no payload de módulos

### Contexto
O contrato legado retorna os módulos diretamente no primeiro nível do JSON consumido por versões antigas do App Desktop.

### Decisão
A nova hierarquia de rotinas foi inserida como campo filho de cada módulo, sem alterar os campos originais na raiz.

### Motivação
Preservar compatibilidade estrita com clientes legados ainda em operação no campo.

### Consequências
Evita quebra de contratos de clientes desktop legados, ao custo de trafegar dados ligeiramente redundantes para clientes novos.
```

---

## 6. Workflow Operacional Obrigatório (7 Etapas)

Ao receber qualquer tarefa de desenvolvimento, siga rigorosamente esta sequência:

```text
1. Identificar o Domínio Raiz (<dominio>/)
       ↓
2. Ler Documentação Existente (<dominio>/README.md e specs)
       ↓
3. Entender a Implementação (código, diffs, testes, migrations)
       ↓
4. Mapear os Contextos Lógicos (regras, fluxos, contratos, dados)
       ↓
5. Criar ou Atualizar as Specs em spec/<implementacao>/
       ↓
6. Criar ou Atualizar o <dominio>/README.md
       ↓
7. Validar a Documentação Contra o Código Final
```

### Etapa 1: Identificar o Domínio
Determinar a pasta raiz que representa o domínio principal da alteração (ex: `src/modules/access-control/`, `app/billing/`, `pkg/auth/`). Se a alteração impactar múltiplos domínios, cada domínio afetado deve ter seu respectivo README revisado.

### Etapa 2: Ler Documentação Existente
Antes de escrever ou alterar documentação:
- Ler o `README.md` do domínio afetado;
- Verificar se já existe pasta em `spec/` para a funcionalidade;
- Verificar specs e decisões anteriores para não contradizer nem duplicar documentação válida.

### Etapa 3: Entender a Implementação
Analisar:
- Requisitos e intenção da tarefa;
- Código implementado ou modificado;
- Migrations, schemas e scripts de banco;
- Contratos de entrada e saída (DTOs, handlers, endpoints);
- Testes unitários, de integração ou e2e;
- Casos de borda (edge cases) e tratamentos de erro.

### Etapa 4: Identificar Contextos Lógicos
Agrupar os aspectos da implementação por afinidade lógica:
- Quais regras de negócio foram criadas ou alteradas?
- Há algoritmos de decisão com fluxos condicionais?
- Há novos contratos de API ou eventos?
- Há persistência ou migrations envolvidas?
- Houve decisões técnicas que exigem registro de contexto?

### Etapa 5: Criar ou Atualizar as Specs
- Criar a pasta `spec/<implementacao>/` utilizando **kebab-case**;
- Criar `overview.md` se houver 2 ou mais arquivos de spec;
- Criar arquivos focados para cada contexto lógico relevante;
- Conectar os arquivos entre si usando links relativos no Markdown.

### Etapa 6: Atualizar o README do Domínio
- Criar ou atualizar `<dominio>/README.md`;
- Resumir as novas capacidades do domínio mantendo o arquivo enxuto;
- Incluir na seção `## Especificações` o link relativo para `spec/<implementacao>/overview.md` ou specs diretas.

### Etapa 7: Validar a Documentação Contra o Código Final
Conferir meticulosamente se:
- Os nomes de campos, colunas, tabelas e funções batem com o código;
- Os payloads de exemplo são válidos e refletem a resposta real;
- As regras de negócio descritas correspondem exatamente à lógica implementada;
- Os edge cases tratados no código foram devidamente documentados.

---

## 7. Diretrizes Críticas de Qualidade

### 7.1. Regra de Atualização Concomitante
<critical>
Documentação faz parte inseparável da implementação. Código alterado com documentação desatualizada é considerado DEFEITO / CÓDIGO INCOMPLETO.
</critical>
Sempre que uma alteração modificar um comportamento documentado anteriormente, atualize a spec e o README no mesmo commit da implementação.

### 7.2. Comportamento Efetivo vs Intenção Inicial
A documentação final DEVE refletir o **comportamento efetivamente implementado**, e NÃO a solicitação inicial da tarefa.
- Se o requisito solicitava "bloquear todas as requisições sem token", mas a implementação real permitiu requisições autenticadas por chave de API legada como exceção transitória, a especificação **deve documentar a regra real com a exceção**.
- Registre divergências relevantes como decisões técnicas (`DEC-XXX`) ou notas de escopo.

### 7.3. Não-Redundância e Links Relativos
Nunca replique blocos inteiros de texto entre arquivos:
- Utilize links relativos: `[veja regras de versão](./version-resolution.md)`;
- Mantenha uma única fonte de verdade para cada regra ou contrato.

### 7.4. Nomenclatura Padrão
- Nomes de pastas e arquivos em **lowercase** e **kebab-case**;
- Idioma alinhado ao repositório (se o projeto utiliza inglês no código e docs, use inglês; se português, use português);
- Nomes conceituais, não estruturais:
  - `spec/<feature>/business-rules.md`
  - `spec/<feature>/module-resolution.md`
  - `spec/<feature>/payload-contract.md`

### 7.5. Profundidade Proporcional
Adequar o volume da documentação à complexidade da entrega:
- **Alteração simples / pontual**:
  ```text
  <dominio>/README.md (atualizado)
  spec/<alteracao>/
  └── behavior.md
  ```
- **Implementação complexa / feature completa**:
  ```text
  <dominio>/README.md (atualizado)
  spec/<implementacao>/
  ├── overview.md
  ├── business-rules.md
  ├── invariants.md
  ├── <algoritmo>-resolution.md
  ├── payload-contract.md
  ├── persistence.md
  ├── compatibility.md
  └── decisions.md
  ```
- ⚠️ **Não criar arquivos vazios ou desnecessários** apenas para preencher uma lista predefinida. Se uma feature não altera banco de dados, não crie `persistence.md`.

---

## 8. Critérios de Conclusão (Checklist Obrigatório)

Antes de considerar a tarefa de implementação concluída, valide todos os itens:

- [ ] A pasta raiz do domínio afetado possui `<dominio>/README.md`.
- [ ] O README do domínio descreve com clareza a responsabilidade atual e foi atualizado cumulativamente.
- [ ] A pasta `spec/<implementacao>/` existe e segue nomenclatura em kebab-case.
- [ ] Se houver múltiplos arquivos de spec, `spec/<implementacao>/overview.md` foi criado como ponto de entrada.
- [ ] As specs foram divididas por contexto lógico (e não por camadas de código).
- [ ] As regras de negócio relevantes foram documentadas com identificadores estáveis (`BR-001`, `BR-002`...).
- [ ] Invariantes do sistema foram explicitadas quando aplicável (`INV-001`...).
- [ ] Decisões técnicas não óbvias foram registradas com contexto e motivação (`DEC-001`...).
- [ ] Contratos de API/payloads têm exemplos reais e semântica detalhada de campos.
- [ ] Alterações de persistência, migrations e valores especiais (ex: `NULL` vs sentinelas) estão documentadas.
- [ ] A documentação reflete o comportamento **efetivo** do código final, e não apenas o requisito inicial.
- [ ] Não há duplicação desnecessária de conteúdo; links relativos foram utilizados.
- [ ] O `<dominio>/README.md` referencia as novas especificações na seção `## Especificações`.

---

## 9. Referências e Exemplos Práticos

Consulte os guias complementares e templates na pasta desta Skill:

- 📑 **Template do README do Domínio**: [`references/domain-readme-template.md`](references/domain-readme-template.md)
- 📝 **Templates de Especificação (Specs)**: [`references/spec-templates.md`](references/spec-templates.md)
- 🏷️ **Guia de Identificadores Estáveis (`BR`, `INV`, `DEC`)**: [`references/rule-identifiers.md`](references/rule-identifiers.md)
- 🚀 **Exemplo Completo (Feature Complexa)**: [`examples/complex-feature.md`](examples/complex-feature.md)
- 💡 **Exemplo Proporcional (Alteração Pontual)**: [`examples/small-change.md`](examples/small-change.md)
