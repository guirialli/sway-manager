# GUI Development Skill

## Objetivo
Instruir agentes na criação e modificação de janelas, popups, OSDs e diálogos no **SwayManager** utilizando PySide6 e Qt6.

## Quando usar
- Ao criar uma nova janela de configuração, popup ou OSD.
- Ao alterar o estilo visual (CSS/QSS) das interfaces do projeto.
- Ao otimizar o ciclo de vida ou consumo de memória de widgets gráficos.

## Specs que devem ser lidas antes
- `specs/gui.md`
- `specs/conventions.md`

## Arquivos importantes
- `src/presentation/gui/app_factory.py`
- `src/presentation/gui/styles.py`
- `src/presentation/gui/windows/`
- `src/presentation/gui/osd/`
- `src/infrastructure/daemon/server/config/gui_config.py`
- `tests/test_freeze_selection_overlay.py`

## Fluxo de trabalho
1. **Definir o Componente Qt**:
   Crie a classe herdando de `QWidget`, `QDialog` ou `QMainWindow`.
2. **Aplicar Banners e Regras Wayland**:
   Garanta que a inicialização chama `setup_qt_environment()`.
3. **Instanciar via `ApplicationFactory`**:
   Sempre execute a interface através de `ApplicationFactory.buildWidget(lambda: CustomWindow())`.
4. **Configurar Atributo de Limpeza**:
   Adicione `widget.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose, True)`.
5. **Conectar Sinal de Destruição**:
   Conecte o sinal `destroyed` para acionar a liberação de memória e `malloc_trim(0)`.

## Regras
- NUNCA crie decorações de janela no lado do cliente (CSD) quando a Sway puder gerenciar as bordas.
- Interfaces OSD transient devem fechar automaticamente usando `QTimer`.
- Janelas GUI devem rodar no processo isolado `SwayManagerGUI`.

## Como validar
- Executar testes de GUI em modo offscreen:
  `PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest discover -s tests -v`
- Testar a abertura da janela diretamente via CLI: `python3 src/gui_main.py <comando>`.

## Erros comuns
- Tentar rodar testes de interface gráfica sem definir a variável de ambiente `QT_QPA_PLATFORM=offscreen`.
- Manter referências fortes a widgets na fábrica de aplicações, impedindo o Garbage Collector de coletá-los.

## Checklist final
- [ ] Interface construída via `ApplicationFactory`.
- [ ] Atributo `WA_DeleteOnClose` configurado.
- [ ] Estilo visual extraído de `styles.py`.
- [ ] Suíte de testes rodando 100% verde em offscreen.
