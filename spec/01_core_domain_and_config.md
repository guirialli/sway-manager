# Spec 01: Core Domain e Repositórios de Sistema (`sway-manager-core`) ⚙️

## 1. Responsabilidade da Crate
Contém o modelo de domínio, entidades, Use Cases e acesso aos subsistemas do Linux (`sysfs`, `swayipc`, arquivos de configuração JSON).

---

## 2. Estrutura de Módulos

```text
crates/sway-manager-core/src/
├── lib.rs
├── domain/
│   ├── mod.rs
│   ├── entities.rs          # Structs: MenuItem, BatteryState, IdleState, PowerProfile, etc.
│   └── traits.rs            # Interfaces/Traits para repositórios
├── infrastructure/
│   ├── mod.rs
│   ├── config.rs            # Leitura/Escrita de ~/.config/sway-manager/config.json
│   ├── battery.rs           # Leitura/Escrita em /sys/bus/platform/drivers/ideapad_acpi/
│   ├── idle.rs              # Gerenciamento de swayidle via pkill / systemd
│   ├── power_profile.rs     # Integração com power-profiles-daemon (D-Bus / sysfs)
│   ├── display.rs           # Comunicação com Sway via swayipc (Troca de monitores)
│   └── wallpaper.rs         # Atualização de ~/.config/sway/config.d/42-wallpaper e symlink
└── use_cases/
    ├── mod.rs
    ├── battery_use_case.rs
    ├── idle_use_case.rs
    ├── power_use_case.rs
    ├── display_use_case.rs
    └── wallpaper_use_case.rs
```

---

## 3. Especificação do Gerenciador de Configurações (`config.rs`)

### Arquivo: `~/.config/sway-manager/config.json`
```json
{
  "wallpaper_folder": "~/Pictures/Wallpapers",
  "screenshot_folder": "~/Pictures/screenshots",
  "theme": "dark"
}
```

### Isolamento Seguro para Testes Unitários
- Se a variável de ambiente `SWAY_MANAGER_TEST_MODE=1` estiver ativa, qualquer leitura/gravação utilizará automaticamente `/tmp/sway_manager_tests/test_config.json`, impedindo alterações nas configurações reais do usuário.

---

## 4. Repositórios de Hardware e Sistema

### A. Bateria Lenovo IdeaPad (`battery.rs`)
- Leitura do estado de conservação em `/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode` ou `/sys/devices/platform/ideapad/conservation_mode`.
- Alternância (0 / 1) via gravação no sysfs (ou com elevação `pkexec` se necessário).

### B. Gerenciamento de SwayIdle (`idle.rs`)
- Identifica se o processo `swayidle` está ativo no sistema.
- Alterna a suspensão ativando/desativando o processo ou enviando sinal.

### C. Modos de Exibição de Monitores (`display.rs`)
- Conecta no socket IPC do Sway utilizando a crate `swayipc`.
- Consulta as saídas ativas (`get_outputs()`) e aplica layouts: `dual`, `pc-only`, `external-only`, `swap`.
