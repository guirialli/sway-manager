# Testing Skill

## Objetivo
Instruir agentes de IA sobre como escrever, organizar, executar e depurar testes unitários e de integração no **SwayManager**.

## Quando usar
- Ao criar novos testes para novos componentes ou Use Cases.
- Ao refatorar código existente e garantir ausência de regressões.
- Ao validar o projeto antes de submeter commits ou Pull Requests.

## Specs que devem ser lidas antes
- `specs/testing.md`
- `specs/conventions.md`

## Arquivos importantes
- `tests/`
- `tests/test_portal.py`
- `tests/test_daemon.py`
- `tests/test_display_repository.py`
- `src/infrastructure/config/json_config_repository.py` (isolamento de teste)

## Fluxo de trabalho
1. **Identificar o Arquivo de Teste Target**:
   Os testes seguem a convenção `tests/test_<modulo>.py`.
2. **Configurar o Ambiente de Execução Headless**:
   Sempre defina as variáveis de ambiente e utilize o Python do venv:
   `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest discover -s tests -v`
3. **Escrever Testes com Mocks Appropriados**:
   - Use `@patch("subprocess.run")` ou `@patch("subprocess.Popen")` para isolar chamadas de sistema.
   - Use `mock_open` para arquivos do sistema `/sys/`.
4. **Verificar Isolamento de Configuração**:
   O `JsonConfigRepository` automaticamente redireciona leituras para `/tmp/sway_manager_tests/test_config.json`.

## Regras
- NUNCA execute testes dependendo de um compositor Sway rodando.
- NUNCA altere os testes existentes apenas para fazê-los passar omitindo asserções.
- Testes que envolvam PySide6/Qt DEVEM rodar em modo `offscreen`.

## Como validar
- Rodar um arquivo individual:
  `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest tests/test_portal.py -v`
- Rodar a suíte completa:
  `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest discover -s tests -v`

## Erros comuns
- Executar os testes utilizando o `python3` global do sistema sem o PySide6 instalado.
- Esquecer a variável `QT_QPA_PLATFORM=offscreen` provocando erros de conexão QPA em ambientes sem display X11/Wayland.

## Checklist final
- [ ] Todos os 96+ testes unitários executados e aprovados (OK).
- [ ] Chamadas de subprocesso mockadas adequadamente.
- [ ] Isolamento de estado e arquivos temporários respeitados.
