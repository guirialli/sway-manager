# Exemplo: Modo `specs`

Cenário: antes de alterar a resolução de rotinas, levantar as regras documentadas
do `controle-acesso-modulos` para não contradizer documentação válida.

## Perguntas

- P1: Quais regras `BR-*` governam disponibilidade de rotinas?
- P2: Quais invariantes `INV-*` e decisões `DEC-*` condicionam a mudança?

## Fan-out (1 scout — alvo conhecido, sem paralelismo artificial)

```json
{
  "context": "Domínio src/modules/access-control/, specs em spec/controle-acesso-modulos/. Objetivo: extrair regras com fonte.",
  "tasks": [
    {
      "name": "SpecsAcesso",
      "agent": "scout",
      "task": "Leia spec/controle-acesso-modulos/overview.md, business-rules.md, invariants.md e decisions.md. Extraia cada BR-*, INV-* e DEC-* com 1 linha de descrição + arquivo-fonte. Não infira regras do código, só transcreva o documentado.",
      "solutionSpace": "leitura dirigida; arquivos conhecidos"
    }
  ]
}
```

## Briefing resultante (resumo)

```markdown
## Briefing de Contexto — specs — controle-acesso-modulos

### Perguntas respondidas
- P1: BR-001 (NULL = não lançado), BR-002 (versão -1 = admin), BR-003 (módulo inativo bloqueia tudo) [fonte: spec/.../business-rules.md]
- P2: INV-003 (cliente normal nunca acessa rotina acima da sua versão); DEC-001 (payload planificado por compatibilidade legada) [fonte: spec/.../invariants.md, decisions.md]

### Handoff
- Próxima etapa: spec-create para atualizar business-rules.md se a mudança alterar BR-002; código ainda por mapear (modo repo)
```
