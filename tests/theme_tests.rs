/// Tests for ThemeColors and builtin theme loading.
use mato::theme::{
    builtin, rgb_to_256, supports_truecolor, supports_truecolor_value, ThemeColors, BUILTIN_THEMES,
};
use ratatui::style::Color;

fn fixed_theme() -> ThemeColors {
    ThemeColors {
        follow_terminal: false,
        bg: [10, 20, 30],
        surface: [40, 50, 60],
        border: [70, 80, 90],
        accent: [100, 110, 120],
        accent2: [130, 140, 150],
        fg: [160, 170, 180],
        fg_dim: [190, 200, 210],
        sel_bg: [220, 230, 240],
    }
}

fn system_theme() -> ThemeColors {
    ThemeColors {
        follow_terminal: true,
        ..fixed_theme()
    }
}

/// Returns the expected Color for RGB values, respecting the current terminal's
/// truecolor support. Tests use this so they are not flaky based on COLORTERM.
fn expected_color(r: u8, g: u8, b: u8) -> Color {
    if supports_truecolor() {
        Color::Rgb(r, g, b)
    } else {
        Color::Indexed(rgb_to_256(r, g, b))
    }
}

// ── follow_terminal = false → best available color ────────────────────────────

#[test]
fn theme_bg_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.bg(), expected_color(10, 20, 30));
}

#[test]
fn theme_surface_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.surface(), expected_color(40, 50, 60));
}

#[test]
fn theme_border_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.border(), expected_color(70, 80, 90));
}

#[test]
fn theme_accent_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.accent(), expected_color(100, 110, 120));
}

#[test]
fn theme_accent2_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.accent2(), expected_color(130, 140, 150));
}

#[test]
fn theme_fg_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.fg(), expected_color(160, 170, 180));
}

#[test]
fn theme_fg_dim_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.fg_dim(), expected_color(190, 200, 210));
}

#[test]
fn theme_sel_bg_returns_color_when_not_follow_terminal() {
    let t = fixed_theme();
    assert_eq!(t.sel_bg(), expected_color(220, 230, 240));
}

// ── follow_terminal = true → Reset ───────────────────────────────────────────

#[test]
fn theme_bg_returns_reset_when_follow_terminal() {
    assert_eq!(system_theme().bg(), Color::Reset);
}

#[test]
fn theme_accent_returns_reset_when_follow_terminal() {
    assert_eq!(system_theme().accent(), Color::Reset);
}

#[test]
fn theme_fg_returns_reset_when_follow_terminal() {
    assert_eq!(system_theme().fg(), Color::Reset);
}

// ── rgb_* accessors return raw arrays ────────────────────────────────────────

#[test]
fn theme_rgb_bg_returns_array() {
    assert_eq!(fixed_theme().rgb_bg(), [10, 20, 30]);
}

#[test]
fn theme_rgb_accent_returns_array() {
    assert_eq!(fixed_theme().rgb_accent(), [100, 110, 120]);
}

#[test]
fn theme_rgb_accent2_returns_array() {
    assert_eq!(fixed_theme().rgb_accent2(), [130, 140, 150]);
}

#[test]
fn theme_rgb_fg_returns_array() {
    assert_eq!(fixed_theme().rgb_fg(), [160, 170, 180]);
}

// ── builtin() ────────────────────────────────────────────────────────────────

#[test]
fn builtin_system_has_follow_terminal_true() {
    assert!(builtin("system").follow_terminal);
}

#[test]
fn builtin_tomato_has_follow_terminal_false() {
    assert!(!builtin("tomato").follow_terminal);
}

#[test]
fn builtin_all_themes_load_without_panic() {
    for name in BUILTIN_THEMES {
        let _ = builtin(name);
    }
}

#[test]
fn builtin_unknown_name_returns_default_theme() {
    // Should not panic; returns some fallback
    let _ = builtin("nonexistent-theme-xyz");
}

#[test]
fn builtin_tomato_accent_is_red_ish() {
    let t = builtin("tomato");
    // accent is #E63946 = (230, 57, 70)
    assert_eq!(t.accent(), expected_color(230, 57, 70));
}

#[test]
fn builtin_nord_bg_is_correct() {
    let t = builtin("nord");
    assert_eq!(t.bg(), expected_color(46, 52, 64));
}

// ── All builtin themes: sanity checks ────────────────────────────────────────

/// Every builtin theme must have valid RGB values (0-255) and not panic.
#[test]
fn all_builtin_themes_have_valid_rgb() {
    for name in BUILTIN_THEMES {
        let t = builtin(name);
        if t.follow_terminal {
            continue; // "system" theme has dummy values, skip rgb check
        }
        for channel in [
            t.bg, t.surface, t.border, t.accent, t.accent2, t.fg, t.fg_dim, t.sel_bg,
        ] {
            // RGB values are u8 so always 0-255; just make sure they exist and are accessible
            let _sum: u32 = channel.iter().map(|&v| v as u32).sum();
        }
    }
}

/// Every builtin name in BUILTIN_THEMES must be handled by builtin() (not fall through to _).
/// The _ arm uses follow_terminal=true; all real themes use follow_terminal=false.
#[test]
fn all_builtin_themes_are_implemented() {
    for name in BUILTIN_THEMES {
        let t = builtin(name);
        if *name == "system" {
            assert!(t.follow_terminal, "system must be follow_terminal=true");
        } else {
            assert!(
                !t.follow_terminal,
                "builtin theme '{}' fell through to wildcard arm (not implemented)",
                name
            );
        }
    }
}

/// One Dark surface must be DARKER than bg (sidebar should be darker than editor).
#[test]
fn one_dark_surface_darker_than_bg() {
    let t = builtin("one-dark");
    let bg_brightness: u32 = t.bg.iter().map(|&v| v as u32).sum();
    let surface_brightness: u32 = t.surface.iter().map(|&v| v as u32).sum();
    assert!(
        surface_brightness < bg_brightness,
        "one-dark surface ({surface_brightness}) must be darker than bg ({bg_brightness})"
    );
}

/// Gruvbox should be warm (red+green channels >= blue, no navy look).
#[test]
fn gruvbox_is_warm_not_cool() {
    let t = builtin("gruvbox");
    let [r, g, b] = t.bg;
    assert!(
        r >= b && g >= b,
        "gruvbox bg should be warm (r={r}, g={g}, b={b})"
    );
}

/// Navy theme should be distinctly blue (blue channel dominant in bg).
#[test]
fn navy_theme_is_blue_dominant() {
    let t = builtin("navy");
    let [r, _g, b] = t.bg;
    assert!(b > r, "navy bg must have blue > red (r={r}, b={b})");
}

// ── Theme merge: partial overrides ──────────────────────────────────────────

use mato::theme::{PartialColors, ThemeFile};

#[test]
fn theme_merge_overrides_accent() {
    let base = builtin("one-dark");
    let custom_accent = [255u8, 0, 128];
    let partial = PartialColors {
        accent: Some(custom_accent),
        ..Default::default()
    };
    // Simulate what load() does
    let file = ThemeFile {
        name: Some("one-dark".into()),
        colors: partial,
    };
    let mut merged = builtin(file.name.as_deref().unwrap_or("system"));
    let o = &file.colors;
    if let Some(v) = o.accent {
        merged.accent = v;
    }
    if let Some(v) = o.bg {
        merged.bg = v;
    }

    assert_eq!(merged.accent, custom_accent, "accent should be overridden");
    assert_eq!(merged.bg, base.bg, "bg should remain from base theme");
}

#[test]
fn theme_merge_all_fields_override() {
    let file = ThemeFile {
        name: Some("gruvbox".into()),
        colors: PartialColors {
            bg: Some([1, 2, 3]),
            surface: Some([4, 5, 6]),
            border: Some([7, 8, 9]),
            accent: Some([10, 11, 12]),
            accent2: Some([13, 14, 15]),
            fg: Some([16, 17, 18]),
            fg_dim: Some([19, 20, 21]),
            sel_bg: Some([22, 23, 24]),
        },
    };
    let mut merged = builtin("gruvbox");
    let o = &file.colors;
    if let Some(v) = o.bg {
        merged.bg = v;
    }
    if let Some(v) = o.surface {
        merged.surface = v;
    }
    if let Some(v) = o.border {
        merged.border = v;
    }
    if let Some(v) = o.accent {
        merged.accent = v;
    }
    if let Some(v) = o.accent2 {
        merged.accent2 = v;
    }
    if let Some(v) = o.fg {
        merged.fg = v;
    }
    if let Some(v) = o.fg_dim {
        merged.fg_dim = v;
    }
    if let Some(v) = o.sel_bg {
        merged.sel_bg = v;
    }

    assert_eq!(merged.bg, [1, 2, 3]);
    assert_eq!(merged.surface, [4, 5, 6]);
    assert_eq!(merged.border, [7, 8, 9]);
    assert_eq!(merged.accent, [10, 11, 12]);
    assert_eq!(merged.accent2, [13, 14, 15]);
    assert_eq!(merged.fg, [16, 17, 18]);
    assert_eq!(merged.fg_dim, [19, 20, 21]);
    assert_eq!(merged.sel_bg, [22, 23, 24]);
}

/// Empty partial (all None) must leave base theme untouched.
#[test]
fn theme_merge_empty_partial_leaves_base_unchanged() {
    let base = builtin("catppuccin");
    let partial = PartialColors::default();
    let mut merged = builtin("catppuccin");
    if let Some(v) = partial.bg {
        merged.bg = v;
    }
    if let Some(v) = partial.surface {
        merged.surface = v;
    }
    assert_eq!(merged.bg, base.bg);
    assert_eq!(merged.surface, base.surface);
    assert_eq!(merged.accent, base.accent);
}

// ── ThemeFile serialization ──────────────────────────────────────────────────

#[test]
fn theme_file_toml_roundtrip_name_only() {
    let input = r#"name = "nord""#;
    let parsed: ThemeFile = toml::from_str(input).unwrap();
    assert_eq!(parsed.name.as_deref(), Some("nord"));
    assert!(parsed.colors.bg.is_none());
}

#[test]
fn theme_file_toml_roundtrip_with_override() {
    let input = r#"
name = "gruvbox"
[colors]
accent = [255, 100, 0]
"#;
    let parsed: ThemeFile = toml::from_str(input).unwrap();
    assert_eq!(parsed.name.as_deref(), Some("gruvbox"));
    assert_eq!(parsed.colors.accent, Some([255, 100, 0]));
    assert!(parsed.colors.bg.is_none());
}

#[test]
fn theme_file_toml_empty_is_default() {
    let parsed: ThemeFile = toml::from_str("").unwrap();
    assert!(parsed.name.is_none());
    assert!(parsed.colors.bg.is_none());
}

// ── supports_truecolor / mosh fallback ──────────────────────────────────────

#[test]
fn supports_truecolor_recognizes_truecolor_value() {
    assert!(supports_truecolor_value(Some("truecolor")));
}

#[test]
fn supports_truecolor_recognizes_24bit_value() {
    assert!(supports_truecolor_value(Some("24bit")));
}

#[test]
fn supports_truecolor_false_when_missing() {
    assert!(!supports_truecolor_value(None));
}

#[test]
fn supports_truecolor_false_for_xterm256() {
    assert!(!supports_truecolor_value(Some("256color")));
}

#[test]
fn load_always_respects_saved_theme_regardless_of_colorterm() {
    // load() must NEVER override the user's saved theme based on COLORTERM.
    // Silently switching to system when COLORTERM is absent breaks persistence
    // for users in tmux, screen, or plain SSH sessions that don't export COLORTERM.
    // The toast warning is shown separately; the theme itself is always loaded.
    let orig = std::env::var("COLORTERM").ok();
    std::env::remove_var("COLORTERM");
    // Whatever theme is on disk, load() returns it unchanged.
    let theme_without_colorterm = mato::theme::load();
    std::env::set_var("COLORTERM", "truecolor");
    let theme_with_colorterm = mato::theme::load();
    match orig {
        Some(v) => std::env::set_var("COLORTERM", v),
        None => std::env::remove_var("COLORTERM"),
    }
    // Both calls must return the same theme — COLORTERM must not influence load().
    assert_eq!(
        theme_without_colorterm.follow_terminal, theme_with_colorterm.follow_terminal,
        "load() must not change theme based on COLORTERM"
    );
}

// ── rgb_to_256 conversion ──────────────────────────────────────────────────

#[test]
fn rgb_to_256_pure_black_maps_to_cube_black() {
    assert_eq!(rgb_to_256(0, 0, 0), 16); // cube entry 0,0,0
}

#[test]
fn rgb_to_256_pure_white_maps_to_cube_white() {
    assert_eq!(rgb_to_256(255, 255, 255), 231); // cube entry 5,5,5
}

#[test]
fn rgb_to_256_grayscale_uses_ramp() {
    // Mid-gray: avg ≈ 128. Ramp index = (128-8)/10 = 12 → 232+12 = 244.
    let idx = rgb_to_256(128, 128, 128);
    assert!(
        idx >= 232 && idx <= 255,
        "expected grayscale ramp, got {idx}"
    );
}

#[test]
fn rgb_to_256_red_maps_to_cube() {
    // Pure red (255,0,0): not near-gray → cube: r6=5, g6=0, b6=0 → 16+36*5 = 196.
    assert_eq!(rgb_to_256(255, 0, 0), 196);
}

#[test]
fn rgb_to_256_blue_maps_to_cube() {
    // Pure blue (0,0,255): not near-gray → cube: r6=0, g6=0, b6=5 → 16+5 = 21.
    assert_eq!(rgb_to_256(0, 0, 255), 21);
}

#[test]
fn color_accessors_never_return_reset_for_non_system_theme() {
    // Regardless of COLORTERM, a non-system theme must return a real color.
    let t = fixed_theme();
    for color in [
        t.bg(),
        t.surface(),
        t.border(),
        t.accent(),
        t.accent2(),
        t.fg(),
        t.fg_dim(),
        t.sel_bg(),
    ] {
        assert_ne!(color, Color::Reset, "expected a real color, got Reset");
    }
}
