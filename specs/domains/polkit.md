# Domain Spec: Polkit Authentication Agent

## 1. Overview & Responsibilities

The **Polkit Domain** replaces the external LXDE-era polkit agent (`lxpolkit`,
currently launched via `polkit-runner.sh`) with a native SwayManager agent split
in two parts:

- **Backend** (`daemon/src/repos/polkit.rs` + resident agent service): registers
  the session on the **system bus** as `org.freedesktop.PolicyKit1.AuthenticationAgent`,
  runs the PAM conversation through the setuid helper, and enforces the fail-secure
  policy learned from CVE-2021-3560.
- **Frontend** (`SwayManagerGUI polkit` + `gui/src/routes/polkit.html`): an
  ephemeral Tauri window per challenge, styled with the Cyberpunk design tokens
  (`specs/gui-theme-tokens.md`), following the GNOME HIG auth-dialog pattern.

`polkitd` itself is untouched — this domain is only the session-side agent.
Reference contracts: `org.freedesktop.PolicyKit1.Authority` and
`org.freedesktop.PolicyKit1.AuthenticationAgent` D-Bus interfaces
(see `specs/adrs/0004-polkit-agent.md` and `specs/todo/polkit-investigacao.md`).

---

## 2. Component Mapping

| Layer | Component | File Path | Responsibility |
|---|---|---|---|
| Domain | `PolkitChallenge`, `HelperEvent`, `polkit::Error` | `daemon/src/repos/polkit.rs` | Pure challenge model, helper-line parser, fail-secure result types |
| Infrastructure | `agent` service (`register`, `serve`, `unregister`) | `daemon/src/repos/polkit.rs` (`agent` module) | `zbus` system-bus object + `RegisterAuthenticationAgent` lifecycle |
| Infrastructure | `session` (`spawn_helper`, `respond`, `cancel`) | `daemon/src/repos/polkit.rs` (`session` module) | setuid-helper/socket conversation driver (PAM text protocol) |
| Infrastructure | CLI routing | `daemon/src/handlers.rs`, `daemon/src/commands.rs`, `daemon/src/main.rs` | `polkit status|agent|dismiss` dispatch; resident agent spawn in `run_daemon` |
| Presentation | GUI entry | `gui/src-tauri/src/main.rs` | `SwayManagerGUI polkit` route, `app_id` fallback, query params (`action_id`, `message`, `identities`) |
| Presentation | Tauri commands | `gui/src-tauri/src/tauri_runtime.rs`, `gui/src-tauri/src/polkit_dialog.rs` | `polkit_submit`, `polkit_cancel`, `polkit_state` invoke handlers |
| Presentation | Frontend | `gui/src/routes/polkit.html` | Auth dialog (password field, identity selector, details expander) |

---

## 3. Subsystem Breakdown

### 3.1 Backend: D-Bus agent (`agent` module)

- Exports `/org/swaymanager/PolicyKit1/AuthenticationAgent` on the **system bus**
  implementing `org.freedesktop.PolicyKit1.AuthenticationAgent`:
  - `BeginAuthentication(s action_id, s message, s icon_name, a{ss} details, s cookie, a(sa{sv}) identities)`
  - `CancelAuthentication(s cookie)`
- Registers with the authority before serving:
  `org.freedesktop.PolicyKit1.Authority.RegisterAuthenticationAgent`
  with subject `("unix-session", {"session-id": <logind id>})` and the current
  locale (`LC_ALL` → `LC_MESSAGES` → `LANG` → `en_US.UTF-8`).
- Resolves the logind session id in order: `XDG_SESSION_ID` →
  `org.freedesktop.login1.Manager.GetSessionByPID(getpid())` → fail-secure abort
  (agent refuses to start without a session).
- One challenge at a time (`m_inProgress` flag, lxqt-policykit parity): a second
  `BeginAuthentication` while one is active is rejected with
  `org.freedesktop.PolicyKit1.Error.Cancelled` and logged; never queued.
- Re-registers when `polkitd` reappears on the bus (`NameOwnerChanged` watch);
  unregisters on shutdown (`UnregisterAuthenticationAgent`).
- Hardening: `prctl(PR_SET_DUMPABLE, 0)` on start (KDE parity); password bytes
  are `zeroize`d after use; `LockedHint` on the logind session vetoes new
  challenges while the screen is locked.

### 3.2 Backend: helper session (`session` module)

- Prefers `/run/polkit/agent-helper.socket` (polkit ≥ 127) and falls back to
  spawning `/usr/lib/polkit-1/polkit-agent-helper-1 <username>` (setuid root).
- Conversation bytes (upstream `polkitagentsession.c` parity):
  - socket path: write `<username>\n<cookie>\n`, then read lines on the same fd.
  - spawn path: argv username, stdin `<cookie>\n`, read lines on child stdout.
  - response to each prompt: `<password>\n` on stdin.
- Line protocol → `HelperEvent` mapping:
  `PAM_PROMPT_ECHO_OFF <t>` → `Request { echo: false }`,
  `PAM_PROMPT_ECHO_ON <t>` → `Request { echo: true }`,
  `PAM_ERROR_MSG <t>` → `ShowError`,
  `PAM_TEXT_INFO <t>` → `ShowInfo`,
  `SUCCESS` → `Completed(true)`, `FAILURE` → `Completed(false)`,
  EOF/HUP/unknown line → `Completed(false)` (fail-secure).
- Cancel path kills the helper (`SIGTERM` + `waitpid`, then `SIGKILL` after a
  grace period) and closes the fds; `BeginAuthentication` replies with
  `org.freedesktop.PolicyKit1.ErrorCancelled`.
- The cookie is only ever written to the helper stdin — never to argv, env,
  logs, or the GUI query string.
### 3.3 Backend: CLI surface

| Command | Path | Behavior |
|---|---|---|
| `polkit status` | sync via socket (`handle_sync`) | One-line agent state: `registered` / `not-registered` + session id; exit 0 |
| `polkit agent` | local/standalone only (`local_invoke`) | Foreground resident agent (for `exec` in Sway config); blocks until signal |
| `polkit dismiss` | interactive via socket | Cancels the in-flight challenge (if any); immediate ack |
| bare `polkit` | sync via socket | Prints usage on stdout |

`polkit agent` never runs through the daemon socket (it *is* the long-lived
process); the daemon instead spawns it detached at startup in `run_daemon`
when no other agent owns the session registration.

### 3.4 Frontend: Tauri dialog

- `SwayManagerGUI polkit --action-id … --message … --identities a,b --pipe … --events …`
  opens `routes/polkit.html` (480×320, frameless + `always_on_top` like the other small repo windows, centered).
- `app_id` fallback via `prgname_for("polkit") = "org.swaymanager.PolkitAuth"`,
  matched by a `for_window [app_id="org.swaymanager.PolkitAuth"] floating enable,
  move position center, sticky enable` rule (documented in `specs/sway-integration.md`).
- Tauri commands (in `invoke_handler`):
  - `polkit_state() -> PolkitDialogState` — challenge payload (message, action id,
    identities, echo mode, info/error banner) for first paint.
  - `polkit_submit(password: String)` — forwards the secret to the backend
    challenge pipe; clears the JS field immediately after invoke.
  - `polkit_cancel()` — aborts the challenge; exits the process.
- Frontend/backend transport (v1): the backend spawns the GUI with an
  inherited-pipe handle; the GUI writes JSON lines
  (`{"password":"…"}`, `{"cancel":true}`) to fd 3 and reads prompt updates from
  fd 4. The cookie never crosses this channel — only prompt/response payloads.
- Visual contract (GNOME HIG auth dialog + Cyberpunk tokens): lock icon + bold
  title (`Autenticação necessária`), action message, identity `<select>` when
  >1 identity, password field with autofocus + visibility toggle + Enter
  inline red error banner that keeps focus without recreating the window.

### 3.5 Installation (`install.sh --install-polkit`)
Opt-in installer (default off): makes the native agent the only active one.

1. Kills rival agents (`polkit-gnome-authentication-agent-1`,
   `lxqt-policykit-agent`, `polkit-kde-authentication-agent-1`,
   `hyprpolkitagent`, `mate-polkit`, `lxpolkit`) and disables/removes the
   legacy `polkit-gnome-authentication-agent-1.service` user unit. Packages
   stay installed (reversible); warns if rivals survive.
2. Rewrites the Sway autostart line (`~/.config/sway/config.d/30-autostart`,
   backup `*.bak-polkit`) from `polkit-runner.sh` to
   `exec $sway-manager polkit agent`.
3. Appends the `for_window [app_id="org.swaymanager.PolkitAuth"]` rule to
   `99-rules` (idempotent) and runs `swaymsg reload` when possible.
4. Smoke test: starts the agent, checks it survives 3 s, kills it; fails the
   install with the agent log on early death.

Transition helpers in the parent repo: `polkit-runner.sh` prefers the native
agent (`~/.config/sway/bin/SwayManager polkit agent`) with legacy fallback,
and `--install-polkit` there delegates to this installer.

---

## 4. Security Invariants (fail-secure)

1. **INV-POLKIT-1**: unknown helper line, EOF, HUP, or spawn failure ⇒
   `Completed(false)`; the D-Bus reply is an error, never success.
2. **INV-POLKIT-2**: agent refuses to start (exit 1 + stderr) when the logind
   session id cannot be resolved.
3. **INV-POLKIT-3**: cookie bytes never appear in argv, env, logs, GUI query
   params, or error strings.
4. **INV-POLKIT-4**: password bytes are zeroized in Rust and cleared in JS
   immediately after forwarding; never logged.
5. **INV-POLKIT-5**: second concurrent challenge is rejected, not queued.
6. **INV-POLKIT-6**: `LockedHint=true` on the session vetoes `BeginAuthentication`.
7. **INV-POLKIT-7**: cancel always kills the helper and closes its fds (no
   orphaned setuid process).

---

## 5. Contracts & Interfaces

### CLI contracts

```bash
SwayManager polkit status    # stdout: "polkit: registered (session 8)\n"; stderr empty; exit 0
SwayManager polkit agent     # foreground service; logs to stderr; exit 1 on startup failure
SwayManager polkit dismiss   # interactive ack (empty stdout/stderr, exit 0)
SwayManager polkit            # stdout: usage text; exit 0
```

### D-Bus contracts (system bus)

```text
service:  org.freedesktop.PolicyKit1           path: /org/freedesktop/PolicyKit1/Authority
method:   RegisterAuthenticationAgent((sa{sv}) subject, s locale, s path)
method:   UnregisterAuthenticationAgent((sa{sv}) subject, s path)
object:   /org/swaymanager/PolicyKit1/AuthenticationAgent
iface:    org.freedesktop.PolicyKit1.AuthenticationAgent
method:   BeginAuthentication(s, s, s, a{ss}, s, a(sa{sv}))
method:   CancelAuthentication(s)
error:    org.freedesktop.PolicyKit1.Error.Cancelled
```

### GUI transport (v1, inherited fds)

```text
GUI argv:   SwayManagerGUI polkit --action-id <id> --message <m> --identities <csv> --pipe <fd3> --events <fd4>
GUI → BE:  {"password":"…"} | {"cancel":true}              (JSON lines on fd 3)
BE → GUI:  {"prompt":"…","echo":false} | {"error":"…"} |
           {"info":"…"} | {"completed":true|false}          (JSON lines on fd 4)
```

---

## 6. Error Handling & Edge Cases

| Condition | System Behavior | User Impact |
|---|---|---|
| `polkitd` not on bus at startup | retry 3× (500 ms), then exit 1 + stderr | agent missing; challenges fail with `NoAgentsAvailable` |
| `polkitd` restarts mid-session | `NameOwnerChanged` → re-register; in-flight challenge cancelled | dialog closes; user retries the action |
| helper binary/socket missing | `Completed(false)` + stderr log | dialog shows "falha de autenticação" inline |
| second challenge arrives | reject with `Error.Cancelled` | first dialog keeps focus; second action errors |
| session locked (`LockedHint`) | reject `BeginAuthentication` | no dialog; action errors until unlock |
| GUI process dies mid-challenge | backend treats as cancel; kills helper | challenge aborted; action errors |
| password with `\n` | rejected client-side (single-line field) | user retypes (PAM line protocol invariant) |

---

## 7. Acceptance Criteria & Verification

- [ ] `cargo test -p sway-manager-daemon polkit` green (parser, session state
      machine, CLI routing, fail-secure paths with `FakeRunner`).
- [ ] `cargo test -p sway-manager-gui polkit` green (dialog state builder,
      query parsing, invoke-handler registration).
- [ ] Socket round-trip test: `polkit status` and `polkit dismiss` through a
      tempdir Unix listener (`daemon/tests/test_daemon.rs` pattern).
- [ ] `cargo clippy --all-targets` warning-free in both crates.
- [ ] Live smoke (Sway session): `SwayManager polkit agent &`, trigger
      `pkexec true`, complete + cancel flows, `LockedHint` veto, kill `-9` GUI
      mid-challenge → helper reaped, no orphans (`ps aux | grep helper`).
- [ ] `swaymsg -t get_tree` shows the dialog floating centered with
      `app_id=org.swaymanager.PolkitAuth`.
