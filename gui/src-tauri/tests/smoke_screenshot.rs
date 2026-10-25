//! Smoke ponta a ponta do screenshot em sessão Sway real.
//!
//! Evolução do `/tmp/dbgproj` throwaway: descobre outputs via `swaymsg`,
//! captura freezes reais (`grim -o`), compõe uma seleção que cruza outputs
//! (`screenshot_compose_save`) e valida o PNG final (dimensões + pixels).
//! Pula graciosamente sem `SWAYSOCK` (CI/headless).

use sway_manager_daemon::process::{CommandRunner, SystemRunner};
use sway_manager_gui::{FrozenScreenDto, Rect};

fn sway_available() -> bool {
    std::env::var("SWAYSOCK").is_ok_and(|sock| !sock.is_empty())
}

fn live_screens(runner: &dyn CommandRunner) -> Vec<FrozenScreenDto> {
    sway_manager_gui::portal_freeze::freeze_screens(runner)
        .into_iter()
        .map(|mut s| {
            // Captura real (paridade `prefreeze_all_outputs`).
            let dst = format!("/tmp/sway-manager-smoke-{}.png", s.name);
            let ok = sway_manager_gui::portal_freeze::freeze_capture(runner, &s.name, &dst)
                .is_ok();
            assert!(ok, "grim -o {} falhou", s.name);
            s.image = Some(dst);
            s
        })
        .collect()
}

#[test]
fn smoke_compose_cross_output_with_real_freezes() {
    if !sway_available() {
        eprintln!("sem SWAYSOCK: pulando smoke de screenshot");
        return;
    }
    let runner = SystemRunner;
    let screens = live_screens(&runner);
    assert!(!screens.is_empty(), "nenhum output ativo");

    // Seleção cruzando a fronteira entre as 2 primeiras telas (ou dentro
    // da única tela em single-monitor).
    let selection = if screens.len() >= 2 {
        let boundary = screens[1].geometry.x;
        Rect { x: boundary - 50, y: 50, w: 100, h: 100 }
    } else {
        let g = screens[0].geometry;
        Rect { x: g.x + 10, y: g.y + 10, w: 100.min(g.w - 20), h: 100.min(g.h - 20) }
    };

    let dir = std::env::temp_dir().join(format!("smoke_shot_{}", std::process::id()));
    let path = sway_manager_gui::screenshot_compose_save(
        &runner,
        selection,
        &screens,
        dir.to_str().unwrap(),
        &std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
    )
    .expect("compose+save deve funcionar em sessão real");

    // PNG válido, não-preto, dimensões coerentes com a seleção × scale.
    let img = image::open(&path).expect("PNG salvo deve decodificar").to_rgba8();
    assert!(img.width() >= 100 && img.height() >= 100, "dimensões {:?}", (img.width(), img.height()));
    let non_black = img.pixels().filter(|p| u16::from(p.0[0]) + u16::from(p.0[1]) + u16::from(p.0[2]) > 12).count();
    assert!(
        non_black > img.width() as usize * img.height() as usize / 100,
        "PNG saiu (quase) todo preto: {path}"
    );
    eprintln!("smoke screenshot OK: {path} ({}x{})", img.width(), img.height());

    let _ = std::fs::remove_dir_all(&dir);
    for screen in &screens {
        if let Some(image) = &screen.image {
            let _ = std::fs::remove_file(image);
        }
    }
}
