# Spec Maintenance Skill

## Objetivo
Instruir agentes de IA sobre como manter a memória arquitetural do projeto (`specs/`) continuamente atualizada, precisa e sincronizada após qualquer modificação no código-fonte, arquitetura, contratos, CLI, GUI ou IPC do **SwayManager**.

## Quando usar
- Sempre que implementar uma nova funcionalidade, Use Case ou repositório.
- Após alterar comandos CLI, argumentos, subcomandos ou contratos de resposta no `stdout`.
- Após modificar o protocolo de comunicação IPC (socket Unix Domain ou mensagens JSON).
- Ao adicionar ou alterar interfaces PySide6 (janelas, OSDs, popups ou temas visuais).
- Ao modificar integrações com o compositor Sway ou ferramentas do Wayland.
- Durante etapas de Code Review antes de finalizar uma tarefa.

## Specs que devem ser lidas antes
- `specs/README.md`
- `specs/architecture.md`
- `specs/project-structure.md`
- `specs/conventions.md`

## Arquivos importantes
- `specs/` (todos os arquivos em `specs/` e `specs/domains/`)
- `specs/README.md` (índice principal da memória do projeto)
- `src/` (código-fonte, a principal fonte de verdade)

## Fluxo de trabalho
1. **Mapear a Área Alterada no Código**:
   Identifique quais arquivos em `src/` foram modificados ou criados. Determine quais especificações existentes em `specs/` cobrem essa funcionalidade.
2. **Consultar o Código Real como Fonte de Verdade**:
   Inspecione a implementação real para extrair a sintaxe exata de comandos, tipos de dados, schemas JSON de requisição/resposta e caminhos de arquivos.
3. **Atualizar / Criar as Specs Correspondentes**:
   - **Novos comandos CLI / Handlers**: Atualize `specs/cli.md` e a spec do domínio afetado em `specs/domains/`.
   - **Alterações de Daemon / IPC**: Atualize `specs/daemon.md` ou `specs/client-server.md`.
   - **Alterações Visuais / Interface Qt**: Atualize `specs/gui.md` ou `specs/theme-design.md`.
   - **Novo Domínio**: Crie `specs/domains/<novo_dominio>.md`.
4. **Manter Linguagem Atemporal (Memória de Longo Prazo)**:
   - EVITE frases temporárias como *"Atualmente estamos alterando X"* ou *"Em breve implementaremos Y"*.
   - PREFIRA afirmações arquiteturais objetivas: *"O subsistema X é responsável por Y e se comunica com Z através de W"*.
5. **Atualizar o Índice (`specs/README.md`)**:
   Se uma nova spec for criada em `specs/`, adicione seu link e descrição no arquivo `specs/README.md`.

## Regras
- O código-fonte executado em `src/` é a fonte primária de verdade. NUNCA documente hipóteses ou ideias não implementadas como fatos.
- NUNCA finalize uma tarefa que altere contratos públicos (CLI, IPC socket, formato de respostas JSON, stdout) sem atualizar as specs correspondentes.
- Novas specs de domínio DEVEM ser salvas dentro de `specs/domains/`.

## Como validar
- Faça uma validação cruzada: confira se todos os nomes de classes, parâmetros, comandos CLI e rotas descritos na spec correspondem exatamente ao código em `src/`.
- Verifique se o arquivo `specs/README.md` aponta corretamente para todas as specs ativas.

## Erros comuns
- Atualizar o código e esquecer de atualizar as specs (causa *documentation drift*).
- Usar linguagem temporária que perde o sentido após algumas alterações no repositório.
- Criar novas specs sem registrar o link no índice `specs/README.md`.

## Checklist final
- [ ] Implementação real em `src/` inspecionada.
- [ ] Specs relevantes em `specs/` ou `specs/domains/` atualizadas.
- [ ] Novas specs (se houver) catalogadas no `specs/README.md`.
- [ ] Estilo atemporal e atestado no código preservado.
