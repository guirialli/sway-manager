# Exemplo: Implementação Completa (`controle-acesso-modulos`)

Este exemplo demonstra uma documentação completa gerada para uma funcionalidade de controle de acesso a módulos e rotinas no domínio `src/modules/access-control/`.

---

## Estrutura de Arquivos Resultante

```text
src/
└── modules/
    └── access-control/
        └── README.md                         ← Camada 1: README do Domínio

spec/
└── controle-acesso-modulos/                  ← Camada 2: Specs Modulares
    ├── overview.md                           ← Ponto de entrada
    ├── business-rules.md                     ← Regras de negócio (BR-001 a BR-003)
    ├── invariants.md                         ← Invariantes (INV-001 a INV-003)
    ├── module-resolution.md                  ← Algoritmo e matriz de decisão
    ├── payload-contract.md                   ← Contrato e semântica de dados
    ├── persistence.md                        ← Tabelas, constraints e migrations
    ├── compatibility.md                      ← Estratégia de compatibilidade
    └── decisions.md                          ← Decisão técnica registrada (DEC-001)
```

---

## 1. README do Domínio: `src/modules/access-control/README.md`

```markdown
# Domínio: Controle de Acesso (`access-control`)

## Responsabilidade
Gerenciar módulos, rotinas, perfis e permissões de usuários, resolvendo a disponibilidade dinâmica de recursos conforme o plano contratado, status de ativação e versão do cliente.

## Contexto
Posiciona-se entre o gateway de autenticação e os módulos de negócio (Financeiro, Fiscal, Estoque), garantindo que apenas rotinas autorizadas e compatíveis com a versão do cliente sejam expostas.

## Conceitos Principais
- **Módulo**: Macrofuncionalidade do sistema (ex: Financeiro, Fiscal).
- **Rotina**: Funcionalidade ou tela específica associada a um módulo (ex: Conciliação Bancária).
- **Versão de Lançamento (`release_version`)**: Versão do produto a partir da qual uma rotina torna-se pública.

## Componentes
- `ModuleResolutionService`: Orquestra a filtragem de módulos e rotinas elegíveis.
- `AccessControlRepository`: Consulta o estado de ativação de módulos e rotinas no banco de dados.
- `AccessController`: Expõe endpoints HTTP para clientes desktop e web.

## Regras Gerais
- Toda rotina pertence obrigatoriamente a um único módulo pai.
- Módulos desativados bloqueiam em cascata todas as suas rotinas.
- Clientes com versão inferior à versão de lançamento de uma rotina não visualizam nem executam o recurso.

## Fluxos Principais
1. **Resolução de Acesso no Login**: Autenticação do usuário → Consulta de módulos contratados → Avaliação de compatibilidade de versões → Devolução de menu autorizado.

## Dependências
- `@sga/database`: Driver e pool de conexões com PostgreSQL.
- `@sga/auth-jwt`: Extração de credenciais do token de acesso.

## Persistência
- Banco: PostgreSQL (`sga_cloud`).
- Tabelas: `access_modules`, `access_routines`, `account_modules`.

## Integrações
- Consumido pelo SGA Desktop Client e SGA Web via REST API.

## Especificações
- [Visão Geral: Controle de Acesso a Módulos](../../../spec/controle-acesso-modulos/overview.md)
- [Regras de Negócio](../../../spec/controle-acesso-modulos/business-rules.md)
- [Algoritmo de Resolução de Rotinas](../../../spec/controle-acesso-modulos/module-resolution.md)
- [Contrato de Payload](../../../spec/controle-acesso-modulos/payload-contract.md)
- [Persistência e Migrações](../../../spec/controle-acesso-modulos/persistence.md)
```

---

## 2. Visão Geral: `spec/controle-acesso-modulos/overview.md`

```markdown
# Controle de Acesso a Módulos e Rotinas

## Objetivo
Implementar controle granular de disponibilização de módulos e rotinas considerando ativação contratual e compatibilidade de versão do cliente.

## Contexto
Anteriormente, a liberação de rotinas dependia apenas do módulo estar ativo. Com lançamentos graduais de novas funcionalidades, clientes desatualizados tentavam carregar rotinas incompatíveis, gerando falhas em runtime.

## Problema Resolvido
Elimina falhas de execução em clientes antigos ao filtrar dinamicamente rotinas cuja versão de lançamento seja superior à versão do cliente solicitante, suportando também homologação de rotinas não lançadas via flag administrativa.

## Escopo
- Filtragem dinâmica de rotinas por versão mínima (`release_version`);
- Suporte a bypass administrativo de versão (`version = -1`);
- Novo payload de retorno com lista aninhada de rotinas autorizadas.

## Fora de Escopo
- Gestão de permissões individuais por usuário (perfil/ACL granular será abordado na próxima etapa).

## Comportamento Geral
Durante a consulta de catálogo de módulos, a aplicação valida se o módulo está ativo e filtra as rotinas elegíveis para a versão do cliente recebida no cabeçalho ou parâmetro.

## Arquitetura da Solução
O `ModuleResolutionService` carrega os módulos da conta, filtra por ativação e aplica a matriz de compatibilidade de versão sobre a coleção de rotinas antes de serializar o JSON de resposta.

## Impactos
Ajuste no formato da resposta HTTP (`GET /api/v1/access-control/modules`). Clientes legados mantêm funcionamento via retrocompatibilidade de campos raiz.

## Compatibilidade
Garantida para versões anteriores do cliente Desktop conforme documentado em [compatibility.md](./compatibility.md).

## Especificações Relacionadas
- [Regras de Negócio](./business-rules.md)
- [Invariantes do Sistema](./invariants.md)
- [Resolução de Módulos e Versões](./module-resolution.md)
- [Contrato do Payload](./payload-contract.md)
- [Persistência](./persistence.md)
- [Decisões Técnicas](./decisions.md)
```

---

## 3. Regras de Negócio: `spec/controle-acesso-modulos/business-rules.md`

```markdown
# Regras de Negócio: Controle de Acesso a Módulos

## BR-001 — Recurso ainda não lançado
Quando `release_version` for `NULL`, a rotina é considerada ainda não lançada oficialmente e fica invisível para versões regulares do cliente.

## BR-002 — Versão administrativa de homologação
Quando a versão recebida na requisição for `-1`, rotinas em desenvolvimento (`release_version = NULL`) podem ser disponibilizadas, desde que o módulo correspondente esteja ativo (`is_active = true`).

## BR-003 — Dependência de ativação do módulo
Uma rotina nunca pode ser disponibilizada quando seu módulo correspondente estiver inativo (`is_active = false`), independentemente da versão do cliente ou de parâmetros administrativos.
```

---

## 4. Invariantes: `spec/controle-acesso-modulos/invariants.md`

```markdown
# Invariantes: Controle de Acesso a Módulos

## INV-001 — Vínculo obrigatório de rotina
Toda rotina deve obrigatoriamente pertencer a um módulo válido e existente no banco de dados (`module_id NOT NULL`).

## INV-002 — Isolamento absoluto de módulo inativo
Nenhuma rotina pode ser disponibilizada quando o módulo correspondente estiver inativo (`is_active = false`).

## INV-003 — Limite estrito de versão de cliente regular
Uma versão regular de cliente (`version != -1`) nunca pode receber ou executar uma rotina cuja `release_version` seja estritamente superior à versão do cliente.
```

---

## 5. Resolução de Módulos e Versões: `spec/controle-acesso-modulos/module-resolution.md`

```markdown
# Algoritmo de Resolução de Módulos e Rotinas

## Fluxo Conceitual
1. Obter a lista de módulos vinculados à conta do cliente solicitante.
2. Descartar módulos cujo status `is_active` seja falso ([BR-003](./business-rules.md#br-003--dependência-de-ativação-do-módulo)).
3. Carregar todas as rotinas filhas dos módulos ativos.
4. Para cada rotina, avaliar a elegibilidade com base em `release_version` e na versão do cliente.
5. Construir a estrutura hierárquica de módulos e rotinas autorizadas.

## Matriz de Decisão de Disponibilidade

| Módulo Ativo | release_version | Versão Informada | Rotina Liberada? | Regra / Invariante |
| :---: | :---: | :---: | :---: | :--- |
| Não | Qualquer | Qualquer | **Não** | [BR-003](./business-rules.md#br-003--dependência-de-ativação-do-módulo) / [INV-002](./invariants.md) |
| Sim | `NULL` | Normal (ex: 2026.1) | **Não** | [BR-001](./business-rules.md#br-001--recurso-ainda-não-lançado) |
| Sim | `NULL` | `-1` (Admin) | **Sim** | [BR-002](./business-rules.md#br-002--versão-administrativa-de-homologação) |
| Sim | `<= Versão Cliente` | Normal | **Sim** | Fluxo normal de compatibilidade |
| Sim | `> Versão Cliente` | Normal | **Não** | [INV-003](./invariants.md#inv-003--limite-estrito-de-versão-de-cliente-regular) |
| Sim | Qualquer valor | `-1` (Admin) | **Sim** | Modo administrativo total |
```

---

## 6. Contrato de Payload: `spec/controle-acesso-modulos/payload-contract.md`

```markdown
# Contrato de Payload: Consulta de Módulos e Rotinas

## Endpoint
`GET /api/v1/access-control/modules?client_version={version}`

## Payload de Resposta (Status 200)
```json
{
  "modules": [
    {
      "id": 10,
      "code": "FIN",
      "name": "Financeiro",
      "is_active": true,
      "routines": [
        {
          "id": 101,
          "code": "FIN_RECONCILIATION",
          "name": "Conciliação Bancária",
          "release_version": "2026.1"
        }
      ]
    }
  ]
}
```

## Dicionário de Campos e Semântica

| Campo | Tipo | Obrigatório | Descrição e Semântica |
| :--- | :--- | :---: | :--- |
| `modules` | Array | Sim | Coleção de módulos ativos para o cliente. |
| `modules[].id` | Integer | Sim | Identificador interno do módulo. |
| `modules[].code` | String | Sim | Sigla mnemônica única do módulo. |
| `modules[].is_active` | Boolean | Sim | Sempre `true` na resposta, pois módulos inativos são filtrados. |
| `modules[].routines` | Array | Sim | Rotinas filtradas e autorizadas conforme versão do cliente. |
| `modules[].routines[].release_version` | String/Null | Sim | Versão oficial de lançamento. `null` se acessado via versão administrativa `-1`. |
```

---

## 7. Persistência: `spec/controle-acesso-modulos/persistence.md`

```markdown
# Persistência: Controle de Acesso a Módulos

## Alterações de Schema

### Tabela `access_routines`
```sql
ALTER TABLE access_routines 
  ADD COLUMN release_version VARCHAR(20) NULL;

CREATE INDEX idx_access_routines_module_release 
  ON access_routines(module_id, release_version);
```

## Semântica dos Valores de `release_version`
- `NULL`: Rotina em desenvolvimento, homologação interna ou rascunho. Não disponível em produção regular.
- `YYYY.M` (ex: `2026.1`): Versão mínima oficial exigida do cliente para execução da rotina.
- `-1`: **Atenção:** Valor reservado exclusivamente para requisições de teste administrativo. **NUNCA DEVE SER PERSISTIDO NO BANCO**.

## Migrações
- Arquivo: `migrations/20260917_add_release_version_to_access_routines.sql`
- Rollback seguro: `ALTER TABLE access_routines DROP COLUMN release_version;`
```

---

## 8. Decisões Técnicas: `spec/controle-acesso-modulos/decisions.md`

```markdown
# Decisões Técnicas: Controle de Acesso a Módulos

## DEC-001 — Manter módulos planificados no payload de resposta

### Contexto
O endpoint original `/api/v1/access-control/modules` retornava uma lista plana de módulos consumida por versões antigas do App Desktop que ainda não implementavam suporte a rotinas aninhadas.

### Decisão
Preservar o array `modules` na raiz e adicionar a lista `routines` como propriedade filha de cada elemento, mantendo inalterados os campos já existentes de cada módulo.

### Motivação
Garantir retrocompatibilidade total com versões antigas do SGA Desktop em operação nos clientes, sem exigir deploys forçados ou gerar exceções de parsing JSON.

### Consequências
Permite coexistência suave entre clientes antigos e clientes modernos na mesma infraestrutura de API.
```

---

## 9. Compatibilidade: `spec/controle-acesso-modulos/compatibility.md`

```markdown
# Compatibilidade: Controle de Acesso a Módulos

## Compatibilidade com Clientes Anteriores
- Clientes que não enviam o parâmetro `client_version` na requisição são tratados com versão padrão de compatibilidade `2025.12`.
- Clientes desktop legados ignoram o campo `routines` desconhecido e continuam exibindo apenas os módulos ativos contratados.
```
