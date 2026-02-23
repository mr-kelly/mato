/// Tests for Vt100Emulator, focusing on the get_screen() rewrite in vt100 0.16
/// where Cell::default() was removed and we now use an explicit None branch.

use mato::emulators::Vt100Emulator;
use mato::terminal_emulator::TerminalEmulator;

// ── Basic rendering ───────────────────────────────────────────────────────────

#[test]
fn vt100_renders_ascii_text() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hello");
    let screen = emu.get_screen(24, 80);
    let row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(row.starts_with("Hello"), "got: {row:?}");
}

#[test]
fn vt100_cursor_advances_after_text() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hi");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.1, 2);
}

#[test]
fn vt100_newline_moves_cursor_down() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"A\r\nB");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.0, 1, "cursor row should be 1 after \\r\\n");
}

#[test]
fn vt100_carriage_return_resets_column() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hello\r");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.1, 0, "cursor col should be 0 after \\r");
}

// ── None-branch: cells outside written area return blank ─────────────────────

#[test]
fn vt100_unwritten_cells_are_space() {
    let mut emu = Vt100Emulator::new(24, 80);
    // Write only to col 0
    emu.process(b"X");
    let screen = emu.get_screen(24, 80);
    // Cells beyond col 0 on row 0 should be space with display_width 1
    let cell = &screen.lines[0].cells[5];
    assert_eq!(cell.ch, ' ', "unwritten cell should be space");
    assert_eq!(cell.display_width, 1);
    assert!(!cell.bold);
    assert!(!cell.reverse);
}

#[test]
fn vt100_empty_emulator_all_cells_are_blank() {
    let emu = Vt100Emulator::new(5, 10);
    let screen = emu.get_screen(5, 10);
    assert_eq!(screen.lines.len(), 5);
    for line in &screen.lines {
        assert_eq!(line.cells.len(), 10);
        for cell in &line.cells {
            assert_eq!(cell.ch, ' ');
            assert_eq!(cell.display_width, 1);
            assert!(!cell.bold);
            assert!(!cell.italic);
            assert!(!cell.underline);
            assert!(!cell.reverse);
            assert!(cell.fg.is_none());
            assert!(cell.bg.is_none());
        }
    }
}

#[test]
fn vt100_get_screen_smaller_than_parser_returns_correct_size() {
    // Parser is 24x80 but we ask for 5x10 — should not panic, returns 5 lines of 10 cells
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hello");
    let screen = emu.get_screen(5, 10);
    assert_eq!(screen.lines.len(), 5);
    assert_eq!(screen.lines[0].cells.len(), 10);
    let row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(row.starts_with("Hello"), "got: {row:?}");
}

// ── Text attributes ───────────────────────────────────────────────────────────

#[test]
fn vt100_bold_attribute_is_set() {
    let mut emu = Vt100Emulator::new(24, 80);
    // SGR 1 = bold on, SGR 0 = reset
    emu.process(b"\x1b[1mBold\x1b[0m");
    let screen = emu.get_screen(24, 80);
    assert!(screen.lines[0].cells[0].bold, "cell should be bold");
    // After reset, next char should not be bold
    emu.process(b"Normal");
    let screen2 = emu.get_screen(24, 80);
    assert!(!screen2.lines[0].cells[4].bold, "cell after reset should not be bold");
}

#[test]
fn vt100_reverse_attribute_is_set() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"\x1b[7mRev\x1b[0m");
    let screen = emu.get_screen(24, 80);
    assert!(screen.lines[0].cells[0].reverse, "cell should be reverse");
}

#[test]
fn vt100_italic_attribute_is_set() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"\x1b[3mItalic\x1b[0m");
    let screen = emu.get_screen(24, 80);
    assert!(screen.lines[0].cells[0].italic, "cell should be italic");
}

#[test]
fn vt100_underline_attribute_is_set() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"\x1b[4mUnder\x1b[0m");
    let screen = emu.get_screen(24, 80);
    assert!(screen.lines[0].cells[0].underline, "cell should be underlined");
}

// ── Colors ────────────────────────────────────────────────────────────────────

#[test]
fn vt100_indexed_fg_color_is_set() {
    let mut emu = Vt100Emulator::new(24, 80);
    // SGR 31 = red (index 1)
    emu.process(b"\x1b[31mR\x1b[0m");
    let screen = emu.get_screen(24, 80);
    assert!(screen.lines[0].cells[0].fg.is_some(), "fg color should be set");
}

#[test]
fn vt100_rgb_fg_color_is_set() {
    let mut emu = Vt100Emulator::new(24, 80);
    // SGR 38;2;255;0;0 = RGB red
    emu.process(b"\x1b[38;2;255;0;0mR\x1b[0m");
    let screen = emu.get_screen(24, 80);
    let fg = screen.lines[0].cells[0].fg;
    assert_eq!(fg, Some(ratatui::style::Color::Rgb(255, 0, 0)));
}

#[test]
fn vt100_default_color_is_none() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"X");
    let screen = emu.get_screen(24, 80);
    assert!(screen.lines[0].cells[0].fg.is_none(), "default fg should be None");
    assert!(screen.lines[0].cells[0].bg.is_none(), "default bg should be None");
}

// ── Wide characters ───────────────────────────────────────────────────────────

#[test]
fn vt100_wide_char_display_width_is_2() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process("中".as_bytes());
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.lines[0].cells[0].ch, '中');
    assert_eq!(screen.lines[0].cells[0].display_width, 2);
}

#[test]
fn vt100_wide_char_spacer_cell_is_blank() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process("中".as_bytes());
    let screen = emu.get_screen(24, 80);
    // vt100 puts a space in the spacer cell (unlike alacritty which uses '\0')
    let spacer = &screen.lines[0].cells[1];
    assert_eq!(spacer.ch, ' ');
}

// ── Screen dimensions ─────────────────────────────────────────────────────────

#[test]
fn vt100_screen_has_correct_dimensions() {
    let emu = Vt100Emulator::new(10, 20);
    let screen = emu.get_screen(10, 20);
    assert_eq!(screen.lines.len(), 10);
    for line in &screen.lines {
        assert_eq!(line.cells.len(), 20);
    }
}

#[test]
fn vt100_cursor_clamped_within_bounds() {
    // Cursor should never exceed (rows-1, cols-1)
    let emu = Vt100Emulator::new(5, 10);
    let screen = emu.get_screen(5, 10);
    assert!(screen.cursor.0 < 5, "cursor row out of bounds");
    assert!(screen.cursor.1 < 10, "cursor col out of bounds");
}

// ── Resize ────────────────────────────────────────────────────────────────────

#[test]
fn vt100_resize_same_size_is_noop() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hello");
    emu.resize(24, 80); // same size — should not clear
    let screen = emu.get_screen(24, 80);
    let row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(row.starts_with("Hello"), "same-size resize should preserve content: {row:?}");
}

#[test]
fn vt100_get_screen_multiple_times_is_stable() {
    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Stable");
    let s1 = emu.get_screen(24, 80);
    let s2 = emu.get_screen(24, 80);
    let r1: String = s1.lines[0].cells.iter().map(|c| c.ch).collect();
    let r2: String = s2.lines[0].cells.iter().map(|c| c.ch).collect();
    assert_eq!(r1, r2, "repeated get_screen should return same content");
}
