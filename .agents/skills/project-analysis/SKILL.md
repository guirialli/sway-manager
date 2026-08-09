# Project Analysis Skill

## Objetivo
Ensinar a futuros agentes de IA como explorar e analisar a base de código do **SwayManager** de maneira segura e abrangente antes de realizar qualquer alteração.

## Quando usar
- Ao iniciar uma nova tarefa ou funcionalidade no repositório SwayManager.
- Ao investigar bugs reportados ou comportamentos inesperados.
- Ao realizar refatorações ou auditorias de código.

## Specs que devem ser lidas antes
- `specs/README.md`
- `specs/architecture.md`
- `specs/project-structure.md`

## Arquivos importantes
- `src/main.py`
- `src/gui_main.py`
- `src/presentation/cli/router.py`
- `src/presentation/cli/handlers.py`
- `src/infrastructure/daemon/daemon_server.py`
- `AGENTS.md`

## Fluxo de trabalho
1. **Verificar Estado e Testes**:
   Execute `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest discover -s tests -v` para confirmar que a base está íntegra.
2. **Mapear Entrypoint e Comando Target**:
   Identifique se a tarefa envolve CLI (`src/main.py`), Daemon (`src/infrastructure/daemon/`), GUI (`src/gui_main.py`) ou Portal (`src/portal/`).
3. **Rastrear Callers e Callees**:
   Trace a cadeia completa:
   `Usuário/CLI -> Router -> Handler -> UseCase -> Repository -> Binário Externo / Sysfs`.
4. **Inspecionar Contratos Atingidos**:
   Verifique se o comando em questão emite JSON para Waybar (`stdout`), altera IPC Unix Socket, ou grava arquivos de configuração.

## Regras
- NUNCA assuma comportamento com base apenas em nomes de arquivos.
- NUNCA altere código sem antes rodar a suíte de testes existente.
- Identifique a fonte de verdade no código real (`src/`) e não em palpites.

## Como validar
- Execute a suíte de testes com a variável `QT_QPA_PLATFORM=offscreen`.
- Teste a chamada CLI correspondente via `--standalone` ou daemon.

## Erros comuns
- Tentar rodar o `python3` do sistema em vez do ambiente `venv/bin/python` onde o PySide6 está instalado.
- Ignorar o sinalizador `QT_QPA_PLATFORM=offscreen` ao rodar testes de interface gráfica.

## Checklist final
- [ ] Testes unitários executados e passando.
- [ ] Fluxo de execução completo rastreado do CLI ao repositório.
- [ ] Specs relevantes consultadas.
