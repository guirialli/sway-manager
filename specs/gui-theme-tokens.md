# Padrão visual SwayManager → tokens de replicação (Tauri/CSS)

Fonte: `src/presentation/gui/styles.py` (+ QSS inline em
`portal/dialog.py`, `windows/config_center.py`, `windows/brightness_popup.py`,
`windows/monitor_swap.py`, `portal/source_card.py`).
Estilo: **macOS HIG** (cards 10–16px, sidebar 230px, scrollbar 8px) +
**Cyberpunk Obsidian & Neon Purple** (dark) / **Apple Classic Light** (light).
Sem `.qss` em disco — tudo gerado por templates Python. Em Tauri: migrar para
variáveis CSS em `:root` (`--window-bg: #0D0B14;` etc.).

Tipografia: `-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'SF Pro Display',
'Segoe UI', Roboto, sans-serif` (`styles.py:6`).

## Tema dark (Cyberpunk Obsidian)

| Token CSS sugerido | Valor | Token sugerido (light) | Valor |
|---|---|---|---|
| `--window-bg` | `#0D0B14` | idem | `#F5F5F7` |
| `--sidebar-bg` | `#13111C` | idem | `#E8E8ED` |
| `--card-bg` | `#191624` | idem | `#FFFFFF` |
| `--card-border` | `rgba(157,78,221,.25)` | idem | `rgba(0,0,0,.08)` |
| `--text-primary` | `#F8F7FF` | idem | `#000000` |
| `--text-secondary` | `#B8B3D0` | idem | `#636366` |
| `--text-muted` | `#7A7596` | idem | `#8E8E93` |
| `--accent` | `#9D4EDD` | idem | `#007AFF` |
| `--accent-hover` | `#C77DFF` | idem | `#0062CC` |
| `--accent-text` | `#FFFFFF` | idem | `#FFFFFF` |
| `--input-bg` | `#14121F` | idem | `#FFFFFF` |
| `--input-border` | `rgba(157,78,221,.35)` | idem | `rgba(0,0,0,.15)` |
| `--button-bg` | `rgba(157,78,221,.12)` | idem | `rgba(0,0,0,.04)` |
| `--button-hover` | `rgba(157,78,221,.25)` | idem | `rgba(0,0,0,.08)` |
| `--button-border` | `rgba(199,125,255,.3)` | idem | `rgba(0,0,0,.1)` |
| `--selection-bg` | `#7B2CBF` | idem | `#007AFF` |
| `--scrollbar` | `rgba(157,78,221,.4)` | idem | `rgba(0,0,0,.2)` |
| `--scrollbar-hover` | `rgba(199,125,255,.7)` | idem | `rgba(0,0,0,.35)` |
| `--osd-bg` | `rgba(13,11,20,235)` | idem | `rgba(240,240,245,220)` |
| `--osd-border` | `rgba(199,125,255,.4)` | idem | `rgba(0,0,0,.12)` |
| `--progress-bg` | `rgba(157,78,221,.2)` | idem | `rgba(0,0,0,.1)` |
| `--progress-chunk` | `#C77DFF` | idem | `#007AFF` |

## Regras de componente

- **Cards** (`.appleCard`): bg `--card-bg`, borda `1px solid --card-border`,
  raio 10–16px. Selecionado: bg `--selection-bg` + borda `2px solid --accent-hover`.
- **Botão primário** (`.primaryButton`): bg sólido `--accent`, hover `--accent-hover`,
  texto `--accent-text`.
- **Botão secundário**: bg `--button-bg`, borda `--button-border`, hover `--button-hover`.
- **OSD HUD**: fundo vidro `--osd-bg`, borda `1px solid --osd-border`, chunk `#C77DFF`;
  auto-close 1,2–1,5 s (`components/base_osd.py:QTimer`, `specs/gui.md:139-140`).
- **Sidebar** (`QListWidget#sidebar`, 230px): bg `--sidebar-bg`; item selecionado bg
  `--accent` (combo usa `--selection-bg`).
- **Inputs**: bg `--input-bg`, borda `--input-border`; focus borda `--accent`.
- **Scrollbar**: 8px, handle `--scrollbar` → hover `--scrollbar-hover`.
- **QPalette dark**: Window `#0D0B14`, Base `#14121F`, AlternateBase/Button `#191624`,
  Highlight `#9D4EDD`, Text `#F8F7FF` (`styles.py:66-88`).

## Geometrias fixas a preservar

- ConfigCenter 920×620, sidebar 230px; MonitorSwap card + revert 15 s;
  BrightnessPopup slider; OSD frameless/top/tool/click-through.

## Cores ad-hoc (não-tokenizadas — decidir na migração)

- `#34C759` sucesso; `#007AFF` retângulo seleção screenshot;
  `rgba(0,0,0,115)` máscara + `rgba(28,28,32,230)` tag dimensões (freeze overlay);
  preview LightDM em Catppuccin Mocha (`#1e1e2e/#313244/#a6adc8/#cdd6f4`)
  em `windows/config_center.py:649-701`.
