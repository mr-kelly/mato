# Release Notes

This directory contains release notes and summaries for Mato versions.

## 📦 Releases

### v0.9.5 (2026-02-24) - Latest
- **[RELEASE_NOTES_v0.9.5.md](RELEASE_NOTES_v0.9.5.md)** - Docs Alignment + Testing Guide + Backlog Refresh
- **[RELEASE_SUMMARY_v0.9.5.md](RELEASE_SUMMARY_v0.9.5.md)** - Technical summary

**Highlights**: dedicated `docs/TESTING.md`, README test-section simplification, and TODO/roadmap synchronization for v0.9.5 planning.

### v0.9.2 (2026-02-23)
- **[RELEASE_NOTES_v0.9.2.md](RELEASE_NOTES_v0.9.2.md)** - Runtime Correctness + Signal UX + Analytics Integration
- **[RELEASE_SUMMARY_v0.9.2.md](RELEASE_SUMMARY_v0.9.2.md)** - Technical summary

**Highlights**: focus-event gating fix (`^[[I`/`^[[O` leakage), bell consume-on-read, spinner timer redraw reliability, toast + jump background dim UX polish, and GA4 integration via `@next/third-parties`.

### v0.9.0 (2026-02-23)
- **[RELEASE_NOTES_v0.9.0.md](RELEASE_NOTES_v0.9.0.md)** - Desk Confirmation UX + Esc Double-Press Reliability + Docs/Website Refresh
- **[RELEASE_SUMMARY_v0.9.0.md](RELEASE_SUMMARY_v0.9.0.md)** - Technical summary

**Highlights**: desk delete yes/no confirmation flow, `Esc-Esc` bell-side-effect fix in Content mode, feature-first README/media refresh, and client/daemon-provider module refactoring with snapshot tests.

### v0.8.1 (2026-02-23)
- **[RELEASE_NOTES_v0.8.1.md](RELEASE_NOTES_v0.8.1.md)** - Jump Mode Viewport Correctness + Startup/Render Stability Polish
- **[RELEASE_SUMMARY_v0.8.1.md](RELEASE_SUMMARY_v0.8.1.md)** - Technical summary

**Highlights**: viewport-aware jump target allocation, digit jump labels, focus-reserved jump key filtering, incremental ScreenDiff path integration, and reduced daemon runtime log noise.

### v0.7.1 (2026-02-22)
- **[RELEASE_NOTES_v0.7.1.md](RELEASE_NOTES_v0.7.1.md)** - Onboarding State/Exit Semantics + Terminal Cleanup Refinement
- **[RELEASE_SUMMARY_v0.7.1.md](RELEASE_SUMMARY_v0.7.1.md)** - Technical summary

**Highlights**: first-run/in-app onboarding mode split, unified runtime screen-state onboarding loop, stronger transition cleanup to eliminate screen residue, and low-risk code quality cleanups.

### v0.7.0 (2026-02-22)
- **[RELEASE_NOTES_v0.7.0.md](RELEASE_NOTES_v0.7.0.md)** - Terminal Rendering Overhaul + Cursor/Bell Pipeline Release
- **[RELEASE_SUMMARY_v0.7.0.md](RELEASE_SUMMARY_v0.7.0.md)** - Technical summary

**Highlights**: Claude cursor visibility fixes via INVERSE rendering, DECTCEM-aware cursor semantics, richer terminal attribute fidelity, bell forwarding, and simplified mode detection.

### v0.6.0 (2026-02-22)
- **[RELEASE_NOTES_v0.6.0.md](RELEASE_NOTES_v0.6.0.md)** - Terminal Resilience + Website Refresh Release
- **[RELEASE_SUMMARY_v0.6.0.md](RELEASE_SUMMARY_v0.6.0.md)** - Technical summary

**Highlights**: PTY auto-respawn after shell exit, proactive desk-switch spawn, faster missing-tab recovery, cursor stability improvements, and a complete website redesign.

### v0.5.0 (2026-02-22)
- **[RELEASE_NOTES_v0.5.0.md](RELEASE_NOTES_v0.5.0.md)** - Jump Mode + System Theme Polish Release
- **[RELEASE_SUMMARY_v0.5.0.md](RELEASE_SUMMARY_v0.5.0.md)** - Technical summary

### v0.4.0 (2026-02-21)
- **[RELEASE_NOTES_v0.4.0.md](RELEASE_NOTES_v0.4.0.md)** - Shell Experience & Performance Release
- **[RELEASE_SUMMARY_v0.4.0.md](RELEASE_SUMMARY_v0.4.0.md)** - Technical summary

### v0.3.0 (2026-02-21)
- **[RELEASE_NOTES_v0.3.0.md](RELEASE_NOTES_v0.3.0.md)** - The Minimalist Release

### v0.2.0 (2026-02-21)
- **[RELEASE_NOTES_v0.2.0.md](RELEASE_NOTES_v0.2.0.md)** - Production Ready Release
- **[RELEASE_SUMMARY_v0.2.0.md](RELEASE_SUMMARY_v0.2.0.md)** - Detailed technical summary
- **[FINAL_SUMMARY.md](FINAL_SUMMARY.md)** - Complete development summary

### v0.1.0 (2026-02-20)
- **[RELEASE_NOTES_v0.1.0.md](RELEASE_NOTES_v0.1.0.md)** - Initial Release

## 📝 Document Types

### RELEASE_NOTES_*.md
Public-facing release announcements:
- What's new
- Installation/upgrade guidance
- Behavior changes

### RELEASE_SUMMARY_*.md
Technical summaries for developers:
- Architecture changes
- Performance/reliability work
- Verification status

### FINAL_SUMMARY.md
Milestone-level historical overview.

---

**Latest Release**: v0.9.5 (2026-02-24)  
**Status**: Stable
