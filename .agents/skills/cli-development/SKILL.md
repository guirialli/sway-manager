# CLI Development Skill

## Objetivo
Guia passo a passo para adicionar ou alterar comandos e subcomandos da interface de linha de comando (`SwayManager`).

## Quando usar
- Ao expor um novo Use Case através da linha de comando.
- Ao modificar argumentos, subcomandos ou sinalizadores CLI existentes.
- Ao alterar o formato de resposta texto/Waybar JSON emitido para o `stdout`.

## Specs que devem ser lidas antes
- `specs/cli.md`
- `specs/client-server.md`

## Arquivos importantes
- `src/presentation/cli/router.py`
- `src/presentation/cli/handlers.py`
- `src/main.py`
- `tests/test_use_cases.py`

## Fluxo de trabalho
1. **Adicionar Roteamento no Router (`router.py`)**:
   - Registre o nome do comando na função `dispatch_cli_command()`.
   - Adicione o caso correspondente na ajuda (`show_help()`).
2. **Implementar o Handler em `CLIHandlers` (`handlers.py`)**:
   - Crie o método estático `handle_<feature>(args: list[str])`.
   - Extraia os argumentos de forma segura usando `ArrayUtils.getSafe(args, index)`.
   - Instancie o repositório de infraestrutura e o Use Case correspondente.
3. **Respeitar o Contrato de Output**:
   - Para integrações Waybar: imprima estritamente o JSON retornado por `.to_waybar_json()`.
   - Para erros: imprima mensagens formatadas em `sys.stderr`.
4. **Atualizar / Adicionar Testes Unitários**:
   Crie testes cobrindo a execução do novo Use Case e Handler.

## Regras
- O `stdout` NUNCA deve receber mensagens de log ou debug quando o comando for do tipo `status` ou `portal`.
- Argumentos CLI devem ser convertidos para minúsculas (`.lower()`) na comparação.
- Argumentos fora do alcance devem ter fallbacks amigáveis tratados por `ArrayUtils`.

## Como validar
- Testar execução do comando em modo standalone:
  `PYTHONPATH=src venv/bin/python src/main.py --standalone <novo_comando>`

## Erros comuns
- Imprimir logs de debug no `stdout` quebram parsers da Waybar e do portal screencast.
- Acessar `args[2]` diretamente sem checar `len(args)` (causa `IndexError`).

## Checklist final
- [ ] Roteador `router.py` e ajuda `--help` atualizados.
- [ ] Handler em `handlers.py` implementado com tratamento de argumentos seguro.
- [ ] Saída `stdout` limpa e validada.
- [ ] Testes unitários atualizados.
