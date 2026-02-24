use crate::client::app::{App, Focus};
use crate::terminal_provider::CursorShape;
use crate::theme::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(super) fn draw_terminal(f: &mut Frame, app: &mut App, area: Rect, t: &ThemeColors) {
    let active = app.focus == Focus::Content;
    let task = &app.offices[app.current_office].desks[app.selected()];
    let tab = task.active_tab_ref();

    let term_bg = t.bg();
    let (ix, iy, iw, ih) = if app.copy_mode {
        f.render_widget(Block::default().style(Style::default().bg(term_bg)), area);
        (area.x, area.y, area.width, area.height)
    } else {
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_type(super::border_type(t, active))
                .title(Span::styled(
                    format!(
                        " {} ",
                        match app.terminal_titles.get(&tab.id) {
                            Some(term_title) if !term_title.is_empty() =>
                                format!("{} : {}", tab.name, term_title),
                            _ => tab.name.clone(),
                        }
                    ),
                    super::title_style(t, active),
                ))
                .border_style(super::border_style(t, active))
                .style(Style::default().bg(term_bg)),
            area,
        );
        (
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        )
    };
    let screen = tab.provider.get_screen(ih, iw);
    let screen_rows = (screen.lines.len() as u16).min(ih);
    // Copy mode: bottom-align so scrollback content is viewable from bottom.
    // Normal mode: also bottom-align when content is shorter than the display area.
    // This handles the Android keyboard resize race: `resize_all_ptys` fires
    // a fire-and-forget Resize while the sync GetScreen fallback may still see
    // the old (smaller) PTY size, returning fewer lines than `ih`. Without
    // bottom-alignment those lines render top-aligned with empty rows below,
    // leaving the cursor visually "stuck in the middle" until the push loop
    // delivers a correctly-sized full screen. Bottom-aligning keeps the cursor
    // pinned near the visual bottom regardless of transient size mismatches.
    let row_base = ih.saturating_sub(screen_rows);

    if screen.bell {
        app.pending_bell = true;
    }

    let buf = f.buffer_mut();
    let bg_style = Style::default().bg(term_bg);

    for row_idx in 0..ih {
        let src_row = if row_idx < row_base {
            None
        } else {
            Some((row_idx - row_base) as usize)
        };
        let by = iy + row_idx;
        if let Some(line) = src_row.and_then(|r| screen.lines.get(r)) {
            let mut bx = ix;
            let bx_end = ix + iw;
            for cell in &line.cells {
                if bx >= bx_end {
                    break;
                }
                if cell.display_width == 0 {
                    continue;
                }
                if let Some(buf_cell) = buf.cell_mut((bx, by)) {
                    // Build style with bitwise modifier accumulation
                    let mut style = Style::default();
                    if let Some(fg) = cell.fg {
                        style = style.fg(fg);
                    }
                    if let Some(bg) = cell.bg {
                        style = style.bg(bg);
                    }
                    let mut mods = Modifier::empty();
                    if cell.bold {
                        mods |= Modifier::BOLD;
                    }
                    if cell.italic {
                        mods |= Modifier::ITALIC;
                    }
                    if cell.underline {
                        mods |= Modifier::UNDERLINED;
                        if let Some(uc) = cell.underline_color {
                            style = style.underline_color(uc);
                        }
                    }
                    if cell.dim {
                        mods |= Modifier::DIM;
                    }
                    if cell.reverse {
                        mods |= Modifier::REVERSED;
                    }
                    if cell.strikethrough {
                        mods |= Modifier::CROSSED_OUT;
                    }
                    if cell.hidden {
                        mods |= Modifier::HIDDEN;
                    }
                    if !mods.is_empty() {
                        style = style.add_modifier(mods);
                    }
                    buf_cell.set_style(style);
                    if cell.ch == '\0' {
                        buf_cell.set_char(' ');
                    } else if let Some(ref zw) = cell.zerowidth {
                        let mut sym = cell.ch.to_string();
                        for &c in zw {
                            sym.push(c);
                        }
                        buf_cell.set_symbol(&sym);
                    } else {
                        buf_cell.set_char(cell.ch);
                    }
                    // Wide chars: reset following continuation cells
                    if cell.display_width > 1 {
                        for dx in 1..cell.display_width as u16 {
                            let cx = bx + dx;
                            if cx < bx_end {
                                if let Some(next_cell) = buf.cell_mut((cx, by)) {
                                    next_cell.reset();
                                }
                            }
                        }
                    }
                }
                bx += cell.display_width as u16;
            }
            // Pad remaining columns with terminal background
            while bx < bx_end {
                if let Some(buf_cell) = buf.cell_mut((bx, by)) {
                    buf_cell.set_char(' ');
                    buf_cell.set_style(bg_style);
                }
                bx += 1;
            }
        } else {
            // Empty row — fill with background
            for col in 0..iw {
                if let Some(buf_cell) = buf.cell_mut((ix + col, by)) {
                    buf_cell.set_char(' ');
                    buf_cell.set_style(bg_style);
                }
            }
        }
    }

    let (cr, cc) = screen.cursor;
    // Hardware cursor is always hidden (terminal.hide_cursor at startup).
    // We use a software cursor overlay rendered in the buffer instead.
    // For Hidden cursor shape (e.g. Claude Code), skip the overlay entirely —
    // the inner TUI app renders its own visual cursor via INVERSE text.
    if !app.copy_mode && ih > 0 && iw > 0 && screen.cursor_shape != CursorShape::Hidden {
        let cursor_row = cr.min(screen_rows.saturating_sub(1));
        let cursor_col = cc.min(iw.saturating_sub(1));
        let visual_cc = screen
            .lines
            .get(cursor_row as usize)
            .map(|line| {
                line.cells
                    .iter()
                    .take(cursor_col as usize)
                    .map(|cell| usize::from(cell.display_width))
                    .sum::<usize>() as u16
            })
            .unwrap_or(cursor_col);
        let cursor_x = ix + visual_cc.min(iw.saturating_sub(1));
        let cursor_y = iy + row_base + cursor_row;

        // Software cursor overlay: render a visible caret in the buffer.
        let line = screen.lines.get(cursor_row as usize);
        let mut glyph = " ".to_string();
        let mut caret_style = Style::default()
            .bg(term_bg)
            .add_modifier(Modifier::REVERSED);
        if let Some(line) = line {
            let mut idx = cc as usize;
            if idx >= line.cells.len() && !line.cells.is_empty() {
                idx = line.cells.len() - 1;
            }
            let mut cell = line.cells.get(idx);
            if matches!(cell, Some(c) if c.ch == '\0') && idx > 0 {
                cell = line.cells.get(idx - 1);
            }
            if let Some(cell) = cell {
                if cell.ch != '\0' {
                    glyph = cell.ch.to_string();
                }
                if let Some(fg) = cell.fg {
                    caret_style = caret_style.fg(fg);
                }
                if let Some(bg) = cell.bg {
                    caret_style = caret_style.bg(bg);
                }
                if cell.bold {
                    caret_style = caret_style.add_modifier(Modifier::BOLD);
                }
                if cell.italic {
                    caret_style = caret_style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline {
                    caret_style = caret_style.add_modifier(Modifier::UNDERLINED);
                }
                caret_style = caret_style.add_modifier(Modifier::REVERSED);
            }
        }
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(glyph, caret_style)])),
            Rect {
                x: cursor_x,
                y: cursor_y,
                width: 1,
                height: 1,
            },
        );
    }
}
