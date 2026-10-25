# Guia de Identificadores Estáveis (`BR`, `INV`, `DEC`)

A rastreabilidade entre documentação e código é um dos pilares da Skill `spec-create`. O uso de identificadores estáveis permite conectar regras de negócio, invariantes e decisões diretamente a testes, commits, PRs e chamados de suporte.

---

## 1. Taxonomia dos Identificadores

| Prefixo | Significado | Finalidade | Exemplo |
| :--- | :--- | :--- | :--- |
| **`BR-XXX`** | *Business Rule* | Regra de negócio ou política condicional de domínio. | `BR-001 — Recurso ainda não lançado` |
| **`INV-XXX`** | *Invariant* | Condição inviolável que nunca pode ser falsa. | `INV-001 — Versão obrigatória em rotina publicada` |
| **`DEC-XXX`** | *Technical Decision* | Registro conciso de decisão arquitetural ou de design. | `DEC-001 — Manter formato planificado no payload` |

---

## 2. Regras de Nomenclatura e Numeração

1. **Sequencial por Implementação**:
   - Cada pasta em `spec/<implementacao>/` possui sua própria numeração sequencial iniciada em `001` (ex: `BR-001`, `BR-002`...).
   - Em sistemas maiores, pode-se prefixar o domínio se necessário (ex: `ACC-BR-001`), mas o formato simples de 3 dígitos (`BR-001`) é o padrão preferencial.
2. **Identificador Imutável**:
   - Uma vez publicado em uma spec, o número de um identificador nunca deve ser alterado ou reaproveitado para outra finalidade.
   - Se uma regra for revogada, marque-a como `[REVOGADA]` na spec e documente o motivo, preservando o número.
3. **Título Autoexplicativo**:
   - Sempre utilize o padrão `PREFIXO-NUMERO — Título Descritivo`.

---

## 3. Como Referenciar no Código e nos Testes

### 3.1. No Código-Fonte (Comentários e Docstrings)
Conecte blocos condicionais críticos diretamente ao ID da regra:

```typescript
// Aplica BR-002: versões administrativas (-1) podem acessar recursos não lançados
if (requestedVersion === -1 && module.isActive) {
  return allowDraftFeatureAccess(routine);
}

// Garante INV-002: isolamento absoluto de rotinas em módulos inativos
if (!module.isActive) {
  throw new ModuleInactiveException(module.id);
}
```

### 3.2. Nos Testes Automatizados
Nomeie ou anote os casos de teste citando o identificador correspondente. Isso torna evidente qual regra está sendo protegida pelo teste:

```typescript
describe('ModuleResolutionService', () => {
  it('deve liberar rotina não lançada quando versão for -1 (BR-002)', () => {
    // ...
  });

  it('deve barrar acesso a qualquer rotina de módulo inativo (INV-002 / BR-003)', () => {
    // ...
  });
});
```

### 3.3. Em Mensagens de Commit e Pull Requests
Vincule alterações ao identificador da regra na descrição do commit:

```text
feat(access-control): implementa resolucao de versao administrativa [BR-002]
fix(access-control): reforca bloqueio de rotinas de modulo inativo [INV-002]
```

### 3.4. Em Outras Especificações (Hiperlinks)
Conecte especificações de fluxos ou contratos às regras de negócio por referência:

```markdown
A regra de filtragem segue [BR-002](./business-rules.md#br-002--versão-administrativa).
```
