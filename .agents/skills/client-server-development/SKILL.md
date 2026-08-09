# Client/Server Development Skill

## Objetivo
Orientar agentes de IA na modificação do protocolo IPC, serialização de mensagens JSON e comunicação via Unix Domain Socket entre a CLI e o Daemon.

## Quando usar
- Ao alterar o formato das mensagens de requisição ou resposta IPC.
- Ao modificar o tempo limite de conexão (timeout) ou política de reconexão do client.
- Ao adicionar novas flags globais passadas do client para o daemon.

## Specs que devem ser lidas antes
- `specs/architecture.md`
- `specs/daemon.md`
- `specs/client-server.md`

## Arquivos importantes
- `src/infrastructure/daemon/daemon_client.py`
- `src/infrastructure/daemon/daemon_server.py`
- `src/presentation/cli/router.py`
- `tests/test_daemon.py`

## Fluxo de trabalho
1. **Atualizar `SwayManagerClient.send_command`**:
   Garanta que a serialização JSON inclui todos os campos necessários no dicionário `{"args": [...]}`.
2. **Atualizar `SwayManagerDaemon._process_request`**:
   Garanta a leitura correta do buffer delimitado por `\n` e o envio da resposta no formato `{"stdout": ..., "stderr": ..., "exit_code": ...}`.
3. **Tratar Ausência do Daemon**:
   Se a conexão falhar ou o socket não existir, retorne `False` graciosamente no client para acionar a mensagem de erro formatada na CLI.
4. **Validar no Teste Unitário**:
   Execute `tests/test_daemon.py`.

## Regras
- Mensagens IPC em ambas as direções DEVEM ser estritamente terminadas com `\n`.
- O payload de stdout retornado ao client NÃO deve conter vazamentos de logs de debug.
- O timeout do client não deve exceder 2.0 segundos para evitar travamento da sessão Sway.

## Como validar
- Testar envio de comando via client em socket falso/mockado.
- Validar tratamento quando o arquivo `.sock` é deletado.

## Erros comuns
- Esquecer o caractere `\n` no final do JSON (faz o `readline()` no receptor travar).
- Tentar reutilizar um socket fechado.

## Checklist final
- [ ] Protocolo JSON em conformidade com a especificação em `specs/client-server.md`.
- [ ] Leitura e escrita com sufixo `\n`.
- [ ] Fallback amigável quando o daemon não está em execução.
