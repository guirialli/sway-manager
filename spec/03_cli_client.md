# Spec 03: Cliente CLI Ultraleve (`sway-manager-cli`) ⚡

## 1. Responsabilidade da Crate
Binário executável de linha de comando (`sway-manager`) construído com a crate `clap`. O cliente converte a entrada do usuário em solicitações IPC em JSON e as envia para o daemon via Unix Socket em **< 1 ms**.

---

## 2. Comandos e Subcomandos Suportados

| Comando CLI | Parâmetros | Descrição |
|---|---|---|
| `sway-manager menu` | `[filtro]` | Dispara o menu Wofi pré-carregado em RAM |
| `sway-manager screenshot` | `area \| fullscreen \| window` | Captura de tela recortada ou completa |
| `sway-manager clipboard` | `menu \| clear \| favorite` | Gerenciador de histórico da área de transferência |
| `sway-manager battery` | `status \| toggle` | Status ou alternância do modo de conservação de bateria |
| `sway-manager idle` | `status \| toggle` | Status ou alternância de inatividade (swayidle) |
| `sway-manager power` | `status \| profile [nome]` | Status ou troca de perfil de energia |
| `sway-manager theme` | `status \| toggle \| dark \| light` | Alternância de tema escuro/claro |
| `sway-manager wallpaper` | `picker \| set [caminho]` | Abre a galeria de wallpapers ou define um papel de parede |
| `sway-manager osd` | `volume [up/down/mute] \| brightness [up/down]` | Exibe popups OSD de volume e brilho |
| `sway-manager monitor` | `swap \| dual \| pc-only` | Alterna layouts de monitores no Sway |
| `sway-manager settings` | N/A | Abre a Central de Controle (GUI) |
| `sway-manager lock` | N/A | Executa o bloqueio de tela (swaylock) |
| `sway-manager daemon log` | `[-f \| --follow]` | Exibe ou acompanha os logs do daemon em tempo real |

---

## 3. Protocolo de Envio e Saída

```rust
// Exemplo de payload enviado ao socket
let req = json!({
    "args": std::env::args().collect::<Vec<String>>()
});
```

- O cliente escreve a linha JSON no socket `$XDG_RUNTIME_DIR/sway-manager.sock`, lê a resposta de confirmação e imprime o `stdout`/`stderr` no terminal antes de sair.
