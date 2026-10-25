# ADR 0004: Agente Polkit Nativo (backend `zbus` + frontend Tauri)

- **Status**: Accepted
- **Date**: 2026-10-25
- **Authors**: SwayManager Core Team

## Context & Problem Statement

O Sway depende de um agente polkit gráfico para autorizar ações `auth_self` /
`auth_admin` vindas de apps GUI (sem TTY). Hoje o autostart usa
`polkit-runner.sh`, que caça binários externos (`polkit-gnome`, `lxqt-policykit`,
`hyprpolkitagent`…): visual inconsistente com o tema Cyberpunk, dependência de
pacotes LXDE/Qt/GNOME, e janelas toplevel X11 sem foco garantido no Sway.
Investigação prévia em `specs/todo/polkit-investigacao.md` (+ briefing web)
mapeou o protocolo exato.

## Decision Drivers

1. **Paridade de protocolo com o upstream**: `RegisterAuthenticationAgent`,
   `BeginAuthentication`/`CancelAuthentication`, conversa PAM via helper setuid
   — tudo verificado no fonte `polkit-org/polkit` (`polkitagentsession.c`,
   `polkitagentlistener.c`).
2. **Fail-secure por construção**: lição do CVE-2021-3560 — qualquer erro de
   identidade/credencial/HUP nega, nunca assume sucesso.
3. **Stack única Rust**: reaproveitar `CommandRunner`/`FakeRunner`, socket IPC,
   e o porte Tauri v2 (ADR 0003) em vez de C/GTK/Qt externos.
4. **UX HIG + tema**: diálogo de auth per GNOME HIG com tokens Cyberpunk,
   foco garantido via regra Sway `for_window` + `app_id` dedicado.

## Considered Options

1. **Manter `polkit-runner.sh` + agente externo** — zero código, mas mantém a
   dependência LXDE/Qt, visual estranho e foco não garantido. Rejeitado.
2. **Bind `libpolkit-agent-1` via FFI/GLib** — protocolo "de graça", mas puxa
   GLib main-loop + GObject para o daemon async Tokio; conflito de runtimes e
   superfície unsafe. Rejeitado.
3. **Agente nativo: `zbus` no daemon + Tauri dialog** (escolhida) — segue o
   modelo do `hyprpolkitagent` (D-Bus direto, sem `libpolkit-agent-1`), com o
   diálogo no padrão já portado do SwayManagerGUI.

## Decision Outcome

Implementar o agente em Rust puro:

- **Backend** (`daemon/src/domains/system/polkit.rs`): objeto D-Bus no system bus via
  `zbus 5` (interface `org.freedesktop.PolicyKit1.AuthenticationAgent` em
  `/org/swaymanager/PolicyKit1/AuthenticationAgent`), registro na authority com
  subject `unix-session`, driver de sessão PAM (socket
  `/run/polkit/agent-helper.socket` → fallback spawn setuid), spawn da GUI por
  desafio, flag `m_inProgress` (1 desafio por vez), `prctl(PR_SET_DUMPABLE, 0)`,
  `zeroize` de segredos, veto por `LockedHint`.
- **Frontend** (`SwayManagerGUI polkit` + `routes/system/polkit.html`): processo
  efêmero por desafio, comandos `polkit_state`/`polkit_submit`/`polkit_cancel`,
  transporte v1 por pipes herdados (JSON lines), cookie nunca cruza para a GUI.
- **CLI**: `polkit status` (sync), `polkit dismiss` (interativo), `polkit agent`
  (foreground local, usado no lugar do `polkit-runner.sh` no autostart).
- **Cutover**: `30-autostart` troca `exec polkit-runner.sh` por
  `exec $sway-manager polkit agent`; `polkit-runner.sh` continua matando agentes
  duplicados legados durante a transição.

### Positive Consequences

- Remove dependência de agentes externos; UX consistente com o sistema.
- Protocolo auditável em Rust puro, com invariantes fail-secure testadas
  (`FakeRunner`, socket round-trip, parser HUP/EOF/unknown-line).
- Reaproveita build/install/testes existentes (`build.sh`, `install.sh`,
  `cargo clippy --all-targets`).

### Negative Consequences / Trade-offs

- Nova dependência `zbus` (+ `zeroize`) no daemon; system-bus em testes exige
  seams (parser/sessão testados sem bus; integração D-Bus só em smoke live).
- Transporte GUI↔backend v1 por pipes herdados (mais simples que socket
  dedicado); cookie fora do canal por design, mas o pipe precisa de testes de
  framing.
- Sem layer-shell no Tauri: foco depende da regra Sway `for_window` +
  `always_on_top` — validar em sessão real; layer-shell nativo fica como
  follow-up.

## Validation & Verification

- Spec do domínio: `specs/domains/polkit.md` §7 (testes + smoke live).
- Smoke obrigatório em sessão Sway real: `pkexec true` (sucesso, senha errada,
  cancelar, GUI morta, `LockedHint`) + `ps` sem helpers órfãos.
