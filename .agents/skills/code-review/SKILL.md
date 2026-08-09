# Code Review Skill

## Objetivo
Instruir agentes na realização de revisões profundas de código em contribuições e modificações no **SwayManager**.

## Quando usar
- Após a conclusão de alterações ou novas funcionalidades.
- Antes de submeter ou aprovar um Pull Request.
- Durante auditorias de segurança e arquitetura.

## Specs que devem ser lidas antes
- `specs/README.md`
- `specs/architecture.md`
- `specs/conventions.md`

## Arquivos importantes
- Todos os arquivos modificados no `git status` ou `git diff`.
- Callsites e dependentes dos módulos alterados.
- Suíte de testes em `tests/`.

## Fluxo de trabalho
1. **Verificação Além do `git diff`**:
   Não analise apenas as linhas alteradas. Identifique quem chama as funções/classes modificadas (Callers) e quem elas chamam (Callees).
2. **Auditoria de Contratos Criticos**:
   - `stdout` limpo em comandos `status` e `portal`.
   - Protocolo JSON delimitado por `\n` no IPC Unix Socket.
   - Resiliência Qt Wayland configurada antes de instanciar `QApplication`.
   - Limpeza de memória (`malloc_trim`, `gc.collect`) em fechamentos GUI.
3. **Auditoria de Especificações (`specs/`)**:
   - Verifique se as alterações no código, contratos ou CLI exigem atualização em `specs/` (siga a skill `spec-maintenance`).
4. **Auditoria de Testes**:
   - Certifique-se de que novos comportamentos foram cobertos por testes unitários.
   - Execute a suíte de testes completa em modo headless.
5. **Auditoria de Desempenho e Recursos**:
   - Verifique ausência de chamadas síncronas bloqueantes na thread principal do Qt.
   - Confirme tratamento de exceções específico sem `except:` genérico que feche o daemon.
## Regras
- REJEITAR qualquer código que insira chamadas de `print()` no `stdout` em manipuladores do portal ou status da Waybar.
- EXIGIR type hints em novos métodos e funções públicas.
- EXIGIR testes unitários para novos repositórios ou Use Cases.

## Como validar
- Rodar suíte de testes: `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest discover -s tests -v`.
- Rodar o script de build `./build.sh` se disponível para validar compilação Nuitka.

## Erros comuns
- Avaliar apenas os arquivos modificados sem checar os callers que dependem de assinaturas alteradas.
- Aprovar alterações que quebrem a execução headless dos testes Qt.

- [ ] Callers e callees verificados.
- [ ] Contratos de `stdout` e IPC preservados.
- [ ] Especificações em `specs/` atualizadas (Skill `spec-maintenance`).
- [ ] Gerenciamento de memória e exceções auditado.
- [ ] Suíte de testes 100% aprovada.
