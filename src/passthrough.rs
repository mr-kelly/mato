/// Passthrough sequence interceptor.
///
/// Parses raw PTY output bytes and splits them into:
/// - Normal bytes (feed to terminal emulator as usual)
/// - APC sequences (`ESC _ ... ESC \`) — Kitty/Sixel/iTerm2 graphics
/// - OSC 7 paths (`ESC ] 7 ; file://host/path BEL/ST`) — working directory
///
/// No heap allocation for the common case (no passthrough sequences).

#[derive(Debug, Default)]
pub struct ParsedPassthrough {
    /// Bytes to feed to the terminal emulator (everything except APC/OSC7).
    pub normal: Vec<u8>,
    /// Complete APC sequences including `\x1b_` prefix and `\x1b\\` suffix.
    /// Each entry is one complete APC (may be a chunk in a multi-chunk transfer).
    pub apc_seqs: Vec<Vec<u8>>,
    /// Decoded working directory paths from OSC 7 (`file://host/path` → `/path`).
    pub osc7_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Normal,
    /// Saw ESC, waiting for next byte to determine sequence type.
    AfterEsc,
    /// Inside APC: `ESC _` ... collecting body.
    InApc,
    /// Inside APC body, saw ESC — waiting for `\` to complete ST.
    InApcAfterEsc,
    /// Inside OSC: `ESC ]` ... collecting body.
    InOsc,
    /// Inside OSC body, saw ESC — waiting for `\` to complete ST.
    InOscAfterEsc,
}

/// Parse `bytes` and split passthrough sequences from normal terminal output.
///
/// The parser is stateless — each call starts fresh. This is intentional: we
/// process PTY output in chunks, and a single sequence will almost never span
/// two separate `read()` calls in practice (OS buffers them together).
/// If a sequence does span chunks, the incomplete part is emitted as normal
/// bytes (the outer terminal ignores unknown APC/OSC sequences safely).
pub fn split_passthrough(bytes: &[u8]) -> ParsedPassthrough {
    let mut out = ParsedPassthrough::default();
    let mut state = State::Normal;
    // Temporary buffers
    let mut apc_buf: Vec<u8> = Vec::new();
    let mut osc_buf: Vec<u8> = Vec::new();

    macro_rules! flush_apc {
        () => {{
            // Complete APC: wrap with ESC_ prefix + ESC\ suffix
            let mut seq = Vec::with_capacity(3 + apc_buf.len());
            seq.extend_from_slice(b"\x1b_");
            seq.extend_from_slice(&apc_buf);
            seq.extend_from_slice(b"\x1b\\");
            out.apc_seqs.push(seq);
            apc_buf.clear();
        }};
    }

    macro_rules! flush_osc {
        () => {{
            if let Some(path) = parse_osc7(&osc_buf) {
                out.osc7_paths.push(path);
            }
            osc_buf.clear();
        }};
    }

    for &b in bytes {
        match state {
            State::Normal => {
                if b == 0x1b {
                    state = State::AfterEsc;
                    // Don't emit ESC yet — might be start of APC/OSC
                } else {
                    out.normal.push(b);
                }
            }
            State::AfterEsc => match b {
                b'_' => {
                    // ESC _ → start APC (Kitty graphics / sixel / iTerm2)
                    state = State::InApc;
                    apc_buf.clear();
                }
                b']' => {
                    // ESC ] → start OSC
                    state = State::InOsc;
                    osc_buf.clear();
                }
                _ => {
                    // Not a sequence we intercept — emit the ESC + this byte as normal
                    out.normal.push(0x1b);
                    out.normal.push(b);
                    state = State::Normal;
                }
            },
            State::InApc => {
                if b == 0x1b {
                    state = State::InApcAfterEsc;
                } else if b == 0x9c {
                    // ST as single byte (C1 control — some terminals use this)
                    flush_apc!();
                    state = State::Normal;
                } else {
                    apc_buf.push(b);
                }
            }
            State::InApcAfterEsc => match b {
                b'\\' => {
                    flush_apc!();
                    state = State::Normal;
                }
                _ => {
                    // ESC inside APC body followed by non-\ — keep going
                    apc_buf.push(0x1b);
                    apc_buf.push(b);
                    state = State::InApc;
                }
            },
            State::InOsc => {
                if b == 0x07 {
                    // BEL terminates OSC
                    flush_osc!();
                    state = State::Normal;
                } else if b == 0x9c {
                    // ST as single byte
                    flush_osc!();
                    state = State::Normal;
                } else if b == 0x1b {
                    state = State::InOscAfterEsc;
                } else {
                    osc_buf.push(b);
                }
            }
            State::InOscAfterEsc => match b {
                b'\\' => {
                    flush_osc!();
                    state = State::Normal;
                }
                _ => {
                    osc_buf.push(0x1b);
                    osc_buf.push(b);
                    state = State::InOsc;
                }
            },
        }
    }

    // If we ended mid-sequence, emit any buffered bytes as normal
    // (incomplete sequence — safe to discard, outer terminal ignores unknown APC/OSC)
    match state {
        State::AfterEsc => out.normal.push(0x1b),
        State::InApc | State::InApcAfterEsc => {
            // Discard incomplete APC — don't feed garbage to the outer terminal
        }
        State::InOsc | State::InOscAfterEsc => {
            // Discard incomplete OSC 7 — just lose the cwd update, not harmful
        }
        State::Normal => {}
    }

    out
}

/// Parse OSC body (between `ESC ]` and ST/BEL).
/// Returns the decoded filesystem path if this is an OSC 7 working-directory sequence.
///
/// OSC 7 format: `7;file://hostname/path` or `7;/path`
fn parse_osc7(osc_body: &[u8]) -> Option<String> {
    let body = std::str::from_utf8(osc_body).ok()?;
    let rest = body.strip_prefix("7;")?;

    // RFC 8089 file URI: file://[host]/path
    if let Some(uri_rest) = rest.strip_prefix("file://") {
        // Strip optional hostname (up to next /)
        let path = if let Some(slash) = uri_rest.find('/') {
            &uri_rest[slash..]
        } else {
            return None; // malformed
        };
        // URL-decode %XX sequences
        Some(percent_decode(path))
    } else if rest.starts_with('/') {
        // Direct path without file:// prefix (non-standard but used by some shells)
        Some(percent_decode(rest))
    } else {
        None
    }
}

/// Minimal percent-decoding for filesystem paths in OSC 7 URIs.
/// Only decodes ASCII percent-encoded bytes; leaves multi-byte UTF-8 intact.
fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                out.push(char::from(h << 4 | l));
                i += 3;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_passthrough_is_identity() {
        let input = b"hello world \x1b[32mfoo\x1b[0m";
        let r = split_passthrough(input);
        assert_eq!(r.normal, input.to_vec());
        assert!(r.apc_seqs.is_empty());
        assert!(r.osc7_paths.is_empty());
    }

    #[test]
    fn apc_extracted_from_normal() {
        // Normal text + APC + more normal text
        let input = b"before\x1b_Ga=T,f=100;AAAA\x1b\\after";
        let r = split_passthrough(input);
        assert_eq!(r.normal, b"beforeafter".to_vec());
        assert_eq!(r.apc_seqs.len(), 1);
        assert_eq!(r.apc_seqs[0], b"\x1b_Ga=T,f=100;AAAA\x1b\\".to_vec());
    }

    #[test]
    fn apc_with_bel_st() {
        // C1 ST (0x9C) terminator
        let input = b"\x1b_Gtest\x9c";
        let r = split_passthrough(input);
        assert_eq!(r.apc_seqs.len(), 1);
        // Output should use ESC\ for consistency
        assert!(r.apc_seqs[0].starts_with(b"\x1b_"));
        assert!(r.apc_seqs[0].ends_with(b"\x1b\\"));
    }

    #[test]
    fn multiple_apc_in_one_chunk() {
        // Two chunked APC sequences (m=1 then m=0)
        let input = b"\x1b_Ga=T,m=1;AAAA\x1b\\\x1b_Gm=0;BBBB\x1b\\";
        let r = split_passthrough(input);
        assert!(r.normal.is_empty());
        assert_eq!(r.apc_seqs.len(), 2);
        assert!(r.apc_seqs[0].contains(&b'1')); // m=1
        assert!(r.apc_seqs[1].contains(&b'0')); // m=0
    }

    #[test]
    fn osc7_file_uri() {
        let input = b"\x1b]7;file://myhostname/home/kelly/code\x07";
        let r = split_passthrough(input);
        assert!(r.normal.is_empty());
        assert_eq!(r.osc7_paths, vec!["/home/kelly/code".to_string()]);
    }

    #[test]
    fn osc7_st_terminator() {
        let input = b"\x1b]7;file://localhost/tmp/test\x1b\\";
        let r = split_passthrough(input);
        assert_eq!(r.osc7_paths, vec!["/tmp/test".to_string()]);
    }

    #[test]
    fn osc7_percent_encoded_path() {
        let input = b"\x1b]7;file://localhost/home/kelly/my%20dir\x07";
        let r = split_passthrough(input);
        assert_eq!(r.osc7_paths, vec!["/home/kelly/my dir".to_string()]);
    }

    #[test]
    fn osc7_mixed_with_normal_and_apc() {
        let input = b"text\x1b]7;file://h/cwd\x07more\x1b_Gfoo\x1b\\end";
        let r = split_passthrough(input);
        assert_eq!(r.normal, b"textmoreend".to_vec());
        assert_eq!(r.osc7_paths, vec!["/cwd".to_string()]);
        assert_eq!(r.apc_seqs.len(), 1);
    }

    #[test]
    fn incomplete_apc_discarded_safely() {
        // Incomplete APC (no terminator) — normal bytes should be clean
        let input = b"before\x1b_Gpartial";
        let r = split_passthrough(input);
        assert_eq!(r.normal, b"before".to_vec());
        assert!(r.apc_seqs.is_empty()); // discarded, not emitted as garbage
    }

    #[test]
    fn non_apc_esc_sequence_passes_through() {
        let input = b"\x1b[32mgreen\x1b[0m";
        let r = split_passthrough(input);
        assert_eq!(r.normal, input.to_vec());
        assert!(r.apc_seqs.is_empty());
    }
}
