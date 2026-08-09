# Daemon Development Skill

## Objetivo
Instruir agentes de IA sobre como modificar ou adicionar novas funcionalidades, comandos e mecanismos de limpeza no daemon `SwayManagerDaemon`.

## Quando usar
- Ao adicionar novos comandos que devem ser processados no daemon.
- Ao alterar o ciclo de vida do socket Unix Domain (`sway-manager-$UID.sock`).
- Ao modificar a estratégia de garbage collection (`perform_gc`) ou tratamento de exceções do servidor.

## Specs que devem ser lidas antes
- `specs/architecture.md`
- `specs/daemon.md`
- `specs/client-server.md`

## Arquivos importantes
- `src/infrastructure/daemon/daemon_server.py`
- `src/infrastructure/daemon/daemon_client.py`
- `src/infrastructure/daemon/server/cli.py`
- `src/infrastructure/daemon/server/gui.py`
- `src/infrastructure/daemon/server/config/gui_config.py`
- `tests/test_daemon.py`

## Fluxo de trabalho
1. **Identificar o Tipo de Comando**:
   - Se for um comando síncrono com saída em texto/JSON (ex: status): registrar em `CliServerHandler.handle_cli_command()`.
   - Se for um comando gráfico/interativo (ex: janela): registrar em `GUI_COMMANDS` em `infrastructure/daemon/server/gui.py`.
2. **Implementar a Lógica com Crash-Protection**:
   Garantir que exceções no handler sejam capturadas e registradas no `AsyncLogger`, sem nunca derrubar o loop principal do Qt (`QCoreApplication`).
3. **Executar Limpeza de Memória**:
   Certificar que `perform_gc()` é chamado no bloco `finally` após a execução do comando.
4. **Adicionar/Atualizar Testes Unitários**:
   Incluir novos casos de teste em `tests/test_daemon.py`.

## Regras
- O daemon principal NUNCA deve instanciar janelas Wayland persistentes em seu próprio processo.
- Comandos gráficos DEVEM ser disparados como subprocessos isolados (`SwayManagerGUI`).
- O socket MUST responder com linha JSON terminada em `\n`.

## Como validar
- Executar testes do daemon:
  `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest tests/test_daemon.py -v`
- Iniciar o daemon localmente e enviar comandos via client.

## Erros comuns
- Bloquear a conexão de socket do client aguardando a conclusão de uma janela GUI (causa timeout).
- Esquecer de desconectar e fechar a conexão de socket.

## Checklist final
- [ ] Handler do comando registrado no servidor CLI ou GUI.
- [ ] Exception handling seguro configurado sem crash do daemon.
- [ ] Chamada de GC no `finally` preservada.
- [ ] Testes unitários atualizados e aprovados.
