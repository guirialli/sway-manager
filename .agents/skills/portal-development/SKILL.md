# Portal Development Skill

## Objetivo
Instruir agentes na manutenção do módulo de compartilhamento de tela e janelas (`src/portal/`) integrado ao `xdg-desktop-portal-wlr`.

## Quando usar
- Ao modificar o diálogo de seleção de fontes de captura (`PortalDialog`).
- Ao alterar os provedores de telas (`SwayOutputsProvider`) ou janelas (`WindowSharingProvider`).
- Ao atualizar o gravador de resultados (`PortalResultWriter`) ou diagnosticador (`PortalDiagnostics`).
- Ao modificar arquivos de configuração do portal (`config_installer.py`).

## Specs que devem ser lidas antes
- `specs/portal.md`
- `specs/gui.md`

## Arquivos importantes
- `src/portal/controller.py`
- `src/portal/result_writer.py`
- `src/portal/outputs_provider.py`
- `src/portal/windows_provider.py`
- `src/presentation/gui/portal/dialog.py`
- `tests/test_portal.py`

## Fluxo de trabalho
1. **Preservar o Contrato de Output**:
   - `PortalResultWriter` é o único componente autorizado a escrever no `stdout`.
   - O output escrito deve ser estritamente `output:<nome>` ou `window:<id>`.
2. **Atualizar Lógica de Seleção**:
   - Modifique `PortalController` ou `PortalDialog`.
3. **Adicionar/Modificar Testes**:
   - Teste todos os fluxos de sucesso e cancelamento no `tests/test_portal.py`.

## Regras
- NUNCA adicione instruções `print()` direcionadas ao `stdout` em qualquer parte do pacote `portal` que não seja o `PortalResultWriter`.
- O cancelamento pelo usuário deve resultar em encerramento com exit code 0 sem escrever nada no `stdout`.
- Tratamento de exceções `SwayNotAvailableError` deve ser mantido no topo do comando.

## Como validar
- Testar o comando de diagnostico do portal:
  `PYTHONPATH=src venv/bin/python src/main.py portal status`
- Testar o seletor em modo interativo de teste:
  `PYTHONPATH=src venv/bin/python src/main.py portal test`
- Executar a suíte de testes unitários do portal:
  `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest tests/test_portal.py -v`

## Erros comuns
- Escrever mensagens de log ou debug no `stdout` quebra a integração com o `xdg-desktop-portal-wlr`.
- Truncar identificadores de janelas ou monitores.

## Checklist final
- [ ] Contrato de stdout preservado sem vazamento de logs.
- [ ] Diálogo Qt acessível via teclado (Enter/Esc).
- [ ] Testes unitários do portal aprovados.
