# Sway Integration Skill

## Objetivo
Instruir agentes de IA na integração de utilitários nativos do ecossistema Wayland e chamadas IPC para o compositor **Sway** / **SwayFX**.

## Quando usar
- Ao criar ou alterar funcionalidades que executam comandos via `swaymsg`.
- Ao integrar novas ferramentas de terminal do ecossistema Wayland (`grim`, `slurp`, `wofi`, `cliphist`, `swaylock`, `swayidle`, `brightnessctl`, `lswt`).
- Ao manipular configurações do Sway em `~/.config/sway/config`.

## Specs que devem ser lidas antes
- `specs/sway-integration.md`
- `specs/domains/display.md`
- `specs/domains/media.md`

## Arquivos importantes
- `src/infrastructure/display/sway_display_repository.py`
- `src/infrastructure/media/grim_slurp_screenshot_repository.py`
- `src/infrastructure/theme/sway_wallpaper_repository.py`
- `src/portal/windows_provider.py`
- `src/portal/outputs_provider.py`

## Fluxo de trabalho
1. **Definir o Protocolo de Chamada**:
   - Para Sway IPC: utilize `swaymsg -t <type> [msg]` com captura de JSON.
   - Para utilitários externos: utilize `subprocess.run(["binario", "args..."], capture_output=True, text=True, check=True)`.
2. **Implementar Tratamento de Erros e Degradabilidade**:
   - Trate `FileNotFoundError` e `subprocess.CalledProcessError`.
   - Se o utilitário estiver ausente no sistema do usuário, forneça um fallback amigável ou levante uma `SwayNotAvailableError`.
3. **Mockar Subprocessos nos Testes Unitários**:
   Utilize `@patch("subprocess.run")` para simular o retorno do `swaymsg` ou binários externos durante os testes.

## Regras
- NUNCA assuma que `swaymsg` ou utilitários externos sempre retornarão sucesso.
- O parsing de saídas JSON do Sway deve ser tolerante a mudanças de versão.
- Subprocessos devem ter timeouts definidos quando aplicável.

## Como validar
- Rodar os testes da infraestrutura correspondente:
  `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest tests/test_display_repository.py -v`
  `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest tests/test_screenshot_repository.py -v`

## Erros comuns
- Executar chamadas bloqueantes de `swaymsg` na thread principal da GUI Qt sem capturar exceções.
- Esquecer de mockar `subprocess.run` nos testes, fazendo com que a suíte tente invocar `swaymsg` em ambiente CI/headful.

## Checklist final
- [ ] Subprocessos envelopados com try/except apropriados.
- [ ] Mock de `subprocess.run` configurado nos testes.
- [ ] Saídas JSON parseadas com segurança.
