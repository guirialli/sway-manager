use crate::cache::ApplicationCache;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sway_manager_core::{
    BatteryUseCase, DisplayLayout, DisplayUseCase, IdleUseCase, PowerProfile, PowerUseCase,
    SystemBatteryRepository, SystemDisplayRepository, SystemIdleRepository, SystemPowerRepository,
    SystemWallpaperRepository, WallpaperUseCase,
};
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct IpcRequest {
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IpcResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub struct CommandDispatcher {
    cache: ApplicationCache,
}

impl CommandDispatcher {
    pub fn new(cache: ApplicationCache) -> Self {
        Self { cache }
    }

    pub fn dispatch(&self, req: IpcRequest) -> IpcResponse {
        if req.args.len() < 2 {
            return IpcResponse {
                stdout: String::new(),
                stderr: "Uso: SwayManager <comando>".to_string(),
                exit_code: 1,
            };
        }

        let cmd = req.args[1].to_lowercase();
        let sub_args = if req.args.len() > 2 { &req.args[2..] } else { &[] };

        match cmd.as_str() {
            "battery" => self.handle_battery(sub_args),
            "idle" => self.handle_idle(sub_args),
            "power" => self.handle_power(sub_args),
            "monitor" => self.handle_monitor(sub_args),
            "wallpaper" => self.handle_wallpaper(sub_args),
            "osd" => self.handle_osd(sub_args),
            "brightness" | "brilho" => self.handle_brightness(sub_args),
            "menu" => self.handle_menu(sub_args),
            "clipboard" | "clip" => self.handle_clipboard(sub_args),
            "screenshot" => self.handle_screenshot(sub_args),
            "lock" => self.handle_lock(),
            "settings" | "config" | "config-center" => self.handle_settings(),
            _ => IpcResponse {
                stdout: String::new(),
                stderr: format!("Comando desconhecido: {}", cmd),
                exit_code: 1,
            },
        }
    }

    fn handle_battery(&self, args: &[String]) -> IpcResponse {
        let repo = SystemBatteryRepository::new();
        let use_case = BatteryUseCase::new(repo);
        let action = args.first().map(|s| s.as_str()).unwrap_or("status");

        match action {
            "toggle" => match use_case.toggle_conservation_mode() {
                Ok(enabled) => IpcResponse {
                    stdout: format!("Modo conservação de bateria: {}", if enabled { "Ativado (~80%)" } else { "Desativado (100%)" }),
                    stderr: String::new(),
                    exit_code: 0,
                },
                Err(err) => IpcResponse {
                    stdout: String::new(),
                    stderr: err,
                    exit_code: 1,
                },
            },
            _ => match use_case.get_status() {
                Ok(state) => {
                    let text = format!("{}% ({})", state.percentage.unwrap_or(0), if state.conservation_mode_enabled { "80%" } else { "100%" });
                    let json_output = json!({
                        "text": text,
                        "alt": if state.conservation_mode_enabled { "conservation" } else { "full" },
                        "tooltip": format!("Bateria: {}%\nStatus: {}\nModo Conservação: {}", state.percentage.unwrap_or(0), state.status, if state.conservation_mode_enabled { "Ativo (~80%)" } else { "Inativo (100%)" }),
                        "percentage": state.percentage.unwrap_or(0)
                    });
                    IpcResponse {
                        stdout: json_output.to_string(),
                        stderr: String::new(),
                        exit_code: 0,
                    }
                }
                Err(err) => IpcResponse {
                    stdout: String::new(),
                    stderr: err,
                    exit_code: 1,
                },
            },
        }
    }

    fn handle_idle(&self, args: &[String]) -> IpcResponse {
        let repo = SystemIdleRepository::new();
        let use_case = IdleUseCase::new(repo);
        let action = args.first().map(|s| s.as_str()).unwrap_or("status");

        match action {
            "toggle" => match use_case.toggle_idle() {
                Ok(inhibited) => IpcResponse {
                    stdout: format!("Inibidor de suspensão: {}", if inhibited { "Ativado" } else { "Desativado" }),
                    stderr: String::new(),
                    exit_code: 0,
                },
                Err(err) => IpcResponse {
                    stdout: String::new(),
                    stderr: err,
                    exit_code: 1,
                },
            },
            _ => match use_case.get_status() {
                Ok(state) => {
                    let json_output = json!({
                        "text": if state.is_inhibited { "☕" } else { "💤" },
                        "alt": if state.is_inhibited { "inhibited" } else { "normal" },
                        "tooltip": format!("Modo Inatividade: {}", if state.is_inhibited { "Inibido (Sem Suspensão)" } else { "Normal (Com Suspensão)" })
                    });
                    IpcResponse {
                        stdout: json_output.to_string(),
                        stderr: String::new(),
                        exit_code: 0,
                    }
                }
                Err(err) => IpcResponse {
                    stdout: String::new(),
                    stderr: err,
                    exit_code: 1,
                },
            },
        }
    }

    fn handle_power(&self, args: &[String]) -> IpcResponse {
        let repo = SystemPowerRepository::new();
        let use_case = PowerUseCase::new(repo);
        let action = args.first().map(|s| s.as_str()).unwrap_or("status");

        match action {
            "toggle" => match use_case.toggle_profile() {
                Ok(new_profile) => IpcResponse {
                    stdout: format!("Perfil de energia alterado para: {}", new_profile.as_str()),
                    stderr: String::new(),
                    exit_code: 0,
                },
                Err(err) => IpcResponse {
                    stdout: String::new(),
                    stderr: err,
                    exit_code: 1,
                },
            },
            "profile" => {
                if let Some(name) = args.get(1) {
                    let profile = PowerProfile::from_str(name);
                    match use_case.set_profile(profile.clone()) {
                        Ok(_) => IpcResponse {
                            stdout: format!("Perfil de energia: {}", profile.as_str()),
                            stderr: String::new(),
                            exit_code: 0,
                        },
                        Err(err) => IpcResponse {
                            stdout: String::new(),
                            stderr: err,
                            exit_code: 1,
                        },
                    }
                } else {
                    IpcResponse {
                        stdout: String::new(),
                        stderr: "Especifique o nome do perfil (performance, balanced, power-saver)".to_string(),
                        exit_code: 1,
                    }
                }
            }
            _ => match use_case.get_profile() {
                Ok(profile) => {
                    let json_output = json!({
                        "text": profile.as_str(),
                        "tooltip": format!("Perfil de Energia: {}", profile.as_str())
                    });
                    IpcResponse {
                        stdout: json_output.to_string(),
                        stderr: String::new(),
                        exit_code: 0,
                    }
                }
                Err(err) => IpcResponse {
                    stdout: String::new(),
                    stderr: err,
                    exit_code: 1,
                },
            },
        }
    }

    fn handle_monitor(&self, args: &[String]) -> IpcResponse {
        let repo = SystemDisplayRepository::new();
        let use_case = DisplayUseCase::new(repo);
        let sub = args.first().map(|s| s.as_str()).unwrap_or("dual");

        let layout = match sub {
            "pc-only" => DisplayLayout::PcOnly,
            "external-only" => DisplayLayout::ExternalOnly,
            "swap" => DisplayLayout::Swap,
            _ => DisplayLayout::Dual,
        };

        match use_case.apply_layout(layout) {
            Ok(_) => IpcResponse {
                stdout: format!("Layout de monitor aplicado: {}", sub),
                stderr: String::new(),
                exit_code: 0,
            },
            Err(err) => IpcResponse {
                stdout: String::new(),
                stderr: err,
                exit_code: 1,
            },
        }
    }

    fn handle_wallpaper(&self, args: &[String]) -> IpcResponse {
        let repo = SystemWallpaperRepository::new();
        let use_case = WallpaperUseCase::new(repo);
        let sub = args.first().map(|s| s.as_str()).unwrap_or("picker");

        if sub == "set" {
            if let Some(path_str) = args.get(1) {
                let path = std::path::Path::new(path_str);
                match use_case.set_wallpaper(path) {
                    Ok(_) => IpcResponse {
                        stdout: format!("Wallpaper definido: {}", path_str),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                    Err(err) => IpcResponse {
                        stdout: String::new(),
                        stderr: err,
                        exit_code: 1,
                    },
                }
            } else {
                IpcResponse {
                    stdout: String::new(),
                    stderr: "Forneça o caminho da imagem".to_string(),
                    exit_code: 1,
                }
            }
        } else {
            // Launch GUI picker in separate thread/task
            std::thread::spawn(|| {
                let _ = sway_manager_gui::show_wallpaper_picker();
            });
            IpcResponse {
                stdout: "Seletor de wallpaper aberto".to_string(),
                stderr: String::new(),
                exit_code: 0,
            }
        }
    }

    fn handle_osd(&self, args: &[String]) -> IpcResponse {
        let target = args.first().map(|s| s.as_str()).unwrap_or("volume");
        let action = args.get(1).map(|s| s.as_str()).unwrap_or("up");

        match target {
            "brightness" | "brilho" => self.handle_brightness(&args[1..]),
            _ => {
                // Adjust volume using pactl/wpctl/pamixer
                let mut val = 50;
                match action {
                    "up" => {
                        let _ = Command::new("pactl").args(["set-sink-volume", "@DEFAULT_SINK@", "+5%"]).status();
                    }
                    "down" => {
                        let _ = Command::new("pactl").args(["set-sink-volume", "@DEFAULT_SINK@", "-5%"]).status();
                    }
                    "mute" => {
                        let _ = Command::new("pactl").args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"]).status();
                    }
                    _ => {}
                }

                // Read current volume
                if let Ok(out) = Command::new("pamixer").arg("--get-volume").output() {
                    if let Ok(parsed) = String::from_utf8_lossy(&out.stdout).trim().parse::<i32>() {
                        val = parsed;
                    }
                }

                let icon = if action == "mute" { "🔇" } else { "🔊" };
                std::thread::spawn(move || {
                    let _ = sway_manager_gui::show_osd("Volume", icon, val);
                });

                IpcResponse {
                    stdout: format!("OSD Volume {}", action),
                    stderr: String::new(),
                    exit_code: 0,
                }
            }
        }
    }

    fn handle_brightness(&self, args: &[String]) -> IpcResponse {
        let action = args.first().map(|s| s.as_str()).unwrap_or("up");
        match action {
            "up" => {
                let _ = Command::new("brightnessctl").args(["set", "+5%"]).status();
            }
            "down" => {
                let _ = Command::new("brightnessctl").args(["set", "5%-"]).status();
            }
            _ => {}
        }

        let mut val = 50;
        if let Ok(out) = Command::new("brightnessctl").arg("info").output() {
            let str_val = String::from_utf8_lossy(&out.stdout);
            if let Some(pos) = str_val.find('(') {
                if let Some(end) = str_val[pos..].find("%") {
                    if let Ok(parsed) = str_val[pos + 1..pos + end].parse::<i32>() {
                        val = parsed;
                    }
                }
            }
        }

        std::thread::spawn(move || {
            let _ = sway_manager_gui::show_osd("Brilho", "☀️", val);
        });

        IpcResponse {
            stdout: format!("OSD Brilho {}", action),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn handle_menu(&self, _args: &[String]) -> IpcResponse {
        let items = self.cache.get_items();
        let mut wofi_input = String::new();
        for item in items {
            wofi_input.push_str(&format!("{}\n", item.name));
        }

        std::thread::spawn(move || {
            let _ = Command::new("wofi")
                .args(["--show", "drun", "--define", "matching=contains"])
                .status();
        });

        IpcResponse {
            stdout: "Menu Wofi aberto".to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn handle_clipboard(&self, args: &[String]) -> IpcResponse {
        let action = args.first().map(|s| s.as_str()).unwrap_or("menu");
        match action {
            "clear" => {
                let _ = Command::new("cliphist").arg("wipe").status();
                IpcResponse {
                    stdout: "Histórico da área de transferência limpo".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                }
            }
            _ => {
                std::thread::spawn(|| {
                    let _ = Command::new("sh")
                        .arg("-c")
                        .arg("cliphist list | wofi --dmenu | cliphist decode | wl-copy")
                        .status();
                });
                IpcResponse {
                    stdout: "Menu de clipboard aberto".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                }
            }
        }
    }

    fn handle_screenshot(&self, args: &[String]) -> IpcResponse {
        let mode = args.first().map(|s| s.as_str()).unwrap_or("area");
        let cmd_str = match mode {
            "full" | "fullscreen" => "grim - | wl-copy",
            "window" => "grim -g \"$(swaymsg -t get_tree | jq -r '.. | select(.focused? == true).rect | \"\\(.x),\\(.y) \\(.width)x\\(.height)\"')\" - | wl-copy",
            _ => "grim -g \"$(slurp)\" - | wl-copy",
        };

        std::thread::spawn(move || {
            let _ = Command::new("sh").arg("-c").arg(cmd_str).status();
        });

        IpcResponse {
            stdout: format!("Screenshot executado (modo: {})", mode),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn handle_lock(&self) -> IpcResponse {
        std::thread::spawn(|| {
            let _ = Command::new("swaylock").args(["-f", "-c", "000000"]).status();
        });
        IpcResponse {
            stdout: "Tela bloqueada".to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn handle_settings(&self) -> IpcResponse {
        std::thread::spawn(|| {
            let _ = sway_manager_gui::show_config_center();
        });
        IpcResponse {
            stdout: "Central de controle aberta".to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }
}
