use mato::protocol::ServerMsg;
use mato::terminal_provider::{CursorShape, ScreenCell, ScreenContent, ScreenLine};

fn make_cell(ch: char) -> ScreenCell {
    ScreenCell {
        ch,
        display_width: 1,
        fg: None,
        bg: None,
        bold: false,
        italic: false,
        underline: false,
        dim: false,
        reverse: false,
        strikethrough: false,
        hidden: false,
        underline_color: None,
        zerowidth: None,
    }
}

fn make_line(s: &str) -> ScreenLine {
    ScreenLine {
        cells: s.chars().map(make_cell).collect(),
    }
}

fn make_screen(lines: &[&str]) -> ScreenContent {
    ScreenContent {
        lines: lines.iter().map(|s| make_line(s)).collect(),
        cursor: (0, 0),
        title: None,
        cursor_shape: CursorShape::Block,
        bell: false,
        focus_events_enabled: false,
        cwd: None,
    }
}

/// Compute diff between old and new screen (mirrors daemon logic)
fn compute_diff(prev: &ScreenContent, content: &ScreenContent) -> Option<ServerMsg> {
    let mut changed: Vec<(u16, ScreenLine)> = Vec::new();
    let max_lines = content.lines.len().max(prev.lines.len());
    for i in 0..max_lines {
        let new_line = content.lines.get(i);
        let old_line = prev.lines.get(i);
        if new_line != old_line {
            if let Some(line) = new_line {
                changed.push((i as u16, line.clone()));
            }
        }
    }
    let meta_changed = content.cursor != prev.cursor
        || content.cursor_shape != prev.cursor_shape
        || content.title != prev.title
        || content.bell
        || content.focus_events_enabled != prev.focus_events_enabled;

    if changed.is_empty() && !meta_changed {
        return None; // unchanged
    }

    // Use diff if fewer than half lines changed (or metadata-only)
    if changed.len() <= max_lines / 2 {
        Some(ServerMsg::ScreenDiff {
            changed_lines: changed,
            cursor: content.cursor,
            cursor_shape: content.cursor_shape.clone(),
            title: content.title.clone(),
            bell: content.bell,
            focus_events_enabled: content.focus_events_enabled,
        })
    } else {
        Some(ServerMsg::Screen {
            tab_id: "test".into(),
            content: content.clone(),
        })
    }
}

/// Apply a ScreenDiff to existing ScreenContent (mirrors client logic)
fn apply_diff(screen: &mut ScreenContent, msg: ServerMsg) {
    match msg {
        ServerMsg::ScreenDiff {
            changed_lines,
            cursor,
            cursor_shape,
            title,
            bell,
            focus_events_enabled,
        } => {
            for (idx, line) in changed_lines {
                let i = idx as usize;
                if i < screen.lines.len() {
                    screen.lines[i] = line;
                }
            }
            screen.cursor = cursor;
            screen.cursor_shape = cursor_shape;
            screen.title = title;
            screen.bell = bell;
            screen.focus_events_enabled = focus_events_enabled;
        }
        ServerMsg::Screen { content, .. } => {
            *screen = content;
        }
        _ => {}
    }
}

// --- Tests ---

#[test]
fn identical_screens_produce_no_diff() {
    let a = make_screen(&["hello", "world", "     "]);
    let b = a.clone();
    assert!(compute_diff(&a, &b).is_none());
}

#[test]
fn single_line_change_produces_diff() {
    let a = make_screen(&["hello", "world", "     "]);
    let mut b = a.clone();
    b.lines[1] = make_line("WORLD");

    let diff = compute_diff(&a, &b).unwrap();
    match &diff {
        ServerMsg::ScreenDiff { changed_lines, .. } => {
            assert_eq!(changed_lines.len(), 1);
            assert_eq!(changed_lines[0].0, 1); // line index
            assert_eq!(changed_lines[0].1, make_line("WORLD"));
        }
        _ => panic!("Expected ScreenDiff, got full Screen"),
    }
}

#[test]
fn cursor_only_change_produces_diff() {
    let a = make_screen(&["hello", "world", "     "]);
    let mut b = a.clone();
    b.cursor = (1, 3);

    let diff = compute_diff(&a, &b).unwrap();
    match &diff {
        ServerMsg::ScreenDiff {
            changed_lines,
            cursor,
            ..
        } => {
            assert!(changed_lines.is_empty());
            assert_eq!(*cursor, (1, 3));
        }
        _ => panic!("Expected ScreenDiff"),
    }
}

#[test]
fn many_lines_changed_falls_back_to_full_screen() {
    let a = make_screen(&["aaaa", "bbbb", "cccc", "dddd"]);
    let b = make_screen(&["AAAA", "BBBB", "CCCC", "dddd"]);
    // 3 out of 4 lines changed (75% > 50%) → full screen
    let diff = compute_diff(&a, &b).unwrap();
    assert!(matches!(diff, ServerMsg::Screen { .. }));
}

#[test]
fn exactly_half_lines_changed_uses_diff() {
    let a = make_screen(&["aaaa", "bbbb", "cccc", "dddd"]);
    let b = make_screen(&["AAAA", "BBBB", "cccc", "dddd"]);
    // 2 out of 4 lines (50% = threshold) → diff
    let diff = compute_diff(&a, &b).unwrap();
    assert!(matches!(diff, ServerMsg::ScreenDiff { .. }));
}

#[test]
fn apply_diff_updates_cached_screen() {
    let mut cached = make_screen(&["hello", "world", "     "]);
    let new_screen = {
        let mut s = cached.clone();
        s.lines[0] = make_line("HELLO");
        s.cursor = (0, 5);
        s
    };

    let diff = compute_diff(&cached, &new_screen).unwrap();
    apply_diff(&mut cached, diff);

    assert_eq!(cached.lines[0], make_line("HELLO"));
    assert_eq!(cached.lines[1], make_line("world")); // unchanged
    assert_eq!(cached.cursor, (0, 5));
}

#[test]
fn apply_full_screen_replaces_entire_cache() {
    let mut cached = make_screen(&["old1", "old2"]);
    let new_screen = make_screen(&["new1", "new2"]);

    let msg = ServerMsg::Screen {
        tab_id: "t".into(),
        content: new_screen.clone(),
    };
    apply_diff(&mut cached, msg);

    assert_eq!(cached.lines[0], make_line("new1"));
    assert_eq!(cached.lines[1], make_line("new2"));
}

#[test]
fn bell_flag_propagates_through_diff() {
    let a = make_screen(&["hello"]);
    let mut b = a.clone();
    b.bell = true;

    let diff = compute_diff(&a, &b).unwrap();
    match &diff {
        ServerMsg::ScreenDiff { bell, .. } => assert!(*bell),
        _ => panic!("Expected ScreenDiff"),
    }

    let mut cached = a.clone();
    apply_diff(&mut cached, diff);
    assert!(cached.bell);
}

#[test]
fn title_change_produces_diff() {
    let a = make_screen(&["hello"]);
    let mut b = a.clone();
    b.title = Some("vim".into());

    let diff = compute_diff(&a, &b).unwrap();
    match &diff {
        ServerMsg::ScreenDiff { title, .. } => {
            assert_eq!(title.as_deref(), Some("vim"));
        }
        _ => panic!("Expected ScreenDiff"),
    }
}

#[test]
fn cursor_shape_change_produces_diff() {
    let a = make_screen(&["hello"]);
    let mut b = a.clone();
    b.cursor_shape = CursorShape::Beam;

    let diff = compute_diff(&a, &b).unwrap();
    match &diff {
        ServerMsg::ScreenDiff { cursor_shape, .. } => {
            assert_eq!(*cursor_shape, CursorShape::Beam);
        }
        _ => panic!("Expected ScreenDiff"),
    }
}

#[test]
fn screen_diff_serializes_with_msgpack() {
    let a = make_screen(&["hello", "world"]);
    let mut b = a.clone();
    b.lines[0] = make_line("HELLO");
    b.cursor = (0, 5);

    let diff = compute_diff(&a, &b).unwrap();

    // Serialize with MessagePack (same as daemon does)
    let bin = rmp_serde::to_vec(&diff).unwrap();
    let full_screen = ServerMsg::Screen {
        tab_id: "t".into(),
        content: b.clone(),
    };
    let full_bin = rmp_serde::to_vec(&full_screen).unwrap();

    // Diff should be significantly smaller than full screen
    assert!(
        bin.len() < full_bin.len(),
        "diff {} bytes should be < full {} bytes",
        bin.len(),
        full_bin.len()
    );

    // Deserialize back
    let deserialized: ServerMsg = rmp_serde::from_slice(&bin).unwrap();
    assert!(matches!(deserialized, ServerMsg::ScreenDiff { .. }));
}

#[test]
fn screen_diff_roundtrip_preserves_content() {
    let original = make_screen(&[
        "line0_original",
        "line1_original",
        "line2_original",
        "line3_original",
    ]);
    let mut updated = original.clone();
    updated.lines[1] = make_line("line1_CHANGED!");
    updated.cursor = (1, 14);
    updated.title = Some("test-title".into());

    let diff = compute_diff(&original, &updated).unwrap();

    // Serialize → deserialize roundtrip
    let bin = rmp_serde::to_vec(&diff).unwrap();
    let restored: ServerMsg = rmp_serde::from_slice(&bin).unwrap();

    // Apply to cached copy
    let mut cached = original.clone();
    apply_diff(&mut cached, restored);

    assert_eq!(cached.lines[0], original.lines[0]); // unchanged
    assert_eq!(cached.lines[1], make_line("line1_CHANGED!")); // updated
    assert_eq!(cached.lines[2], original.lines[2]); // unchanged
    assert_eq!(cached.lines[3], original.lines[3]); // unchanged
    assert_eq!(cached.cursor, (1, 14));
    assert_eq!(cached.title, Some("test-title".into()));
}

// ── focus_events_enabled + bell robustness ────────────────────────────────────

#[test]
fn screen_content_defaults_have_no_bell_no_focus_events() {
    let sc = ScreenContent::default();
    assert!(!sc.bell, "default bell must be false");
    assert!(
        !sc.focus_events_enabled,
        "default focus_events_enabled must be false"
    );
}

#[test]
fn focus_events_enabled_propagates_through_diff() {
    let a = make_screen(&["hello"]);
    let mut b = a.clone();
    b.focus_events_enabled = true;

    let diff = compute_diff(&a, &b).unwrap();
    match &diff {
        ServerMsg::ScreenDiff {
            focus_events_enabled,
            ..
        } => assert!(
            *focus_events_enabled,
            "diff should carry focus_events_enabled=true"
        ),
        _ => panic!("Expected ScreenDiff"),
    }

    let mut cached = a.clone();
    apply_diff(&mut cached, diff);
    assert!(
        cached.focus_events_enabled,
        "cache must have focus_events_enabled=true after applying diff"
    );
}

#[test]
fn focus_events_enabled_false_clears_after_true_via_diff() {
    let base = make_screen(&["hello"]);
    let mut enabled = base.clone();
    enabled.focus_events_enabled = true;

    let diff1 = compute_diff(&base, &enabled).unwrap();
    let mut cached = base.clone();
    apply_diff(&mut cached, diff1);
    assert!(cached.focus_events_enabled);

    // App disables focus tracking (e.g. exits vim)
    let mut disabled = enabled.clone();
    disabled.focus_events_enabled = false;
    disabled.lines[0] = make_line("bye"); // force non-empty diff
    let diff2 = compute_diff(&enabled, &disabled).unwrap();
    apply_diff(&mut cached, diff2);
    assert!(
        !cached.focus_events_enabled,
        "focus_events_enabled must be cleared after diff with false"
    );
}

#[test]
fn bell_cleared_by_subsequent_diff_without_bell() {
    // Bell arrives in diff1; diff2 has no bell → cached bell must become false.
    let a = make_screen(&["hello"]);
    let mut b = a.clone();
    b.bell = true;

    let diff1 = compute_diff(&a, &b).unwrap();
    let mut cached = a.clone();
    apply_diff(&mut cached, diff1);
    assert!(cached.bell, "bell should be true after first diff");

    let mut c = b.clone();
    c.bell = false;
    c.lines[0] = make_line("world"); // force a real diff so compute_diff returns Some
    let diff2 = compute_diff(&b, &c).unwrap();
    apply_diff(&mut cached, diff2);
    assert!(!cached.bell, "bell must be false after diff without bell");
}
