# Testing

Mato test coverage lives in Rust test suites (unit, integration, snapshot, protocol, emulator, and daemon/client behavior).

## Run Tests

```bash
cargo test
```

## Snapshot Tests

```bash
INSTA_UPDATE=always cargo test --test ui_snapshot_tests
cargo insta review
```

## Notes

- Snapshot tests use `insta` with ratatui `TestBackend`.
- Some environment-dependent tests (for example truecolor detection) are sensitive to local env vars.
