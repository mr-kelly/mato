# Release Guide

## Preparing a Release

### 1. Update Version

Edit `Cargo.toml`:
```toml
version = "0.1.0"  # Update this
```

### 2. Update CHANGELOG.md

Move items from `[Unreleased]` to a new version section:
```markdown
## [0.1.0] - 2026-02-21

### Added
- Feature 1
- Feature 2
```

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.1.0"
git push
```

### 4. Create and Push Tag

```bash
git tag v0.1.0
git push origin v0.1.0
```

This will trigger the GitHub Actions workflow to:
- Build binaries for all platforms
- Create a GitHub release
- Upload binaries and checksums

### 5. Update Homebrew Formula

After the release is created:

1. Use your tap repository (recommended: `mr-kelly/homebrew-tap`)
2. Download release checksums:
   ```bash
   curl -fsSL https://github.com/mr-kelly/mato/releases/download/vX.Y.Z/checksums.txt
   ```
3. Update `Formula/mato.rb` in the tap repo with:
   - release URLs
   - matching SHA256 values
4. Commit and push in the tap repo:
   ```bash
   git add Formula/mato.rb
   git commit -m "mato X.Y.Z"
   git push
   ```

### 6. Test Installation

```bash
# Test install script
curl -fsSL http://mato.sh/install.sh | bash

# Test Homebrew
brew tap mr-kelly/tap
brew install mato

# Test binary
mato --version
```

## Release Checklist

- [ ] Version bumped in Cargo.toml
- [ ] CHANGELOG.md updated
- [ ] All tests pass (`cargo test`)
- [ ] Binary builds locally (`cargo build --release`)
- [ ] Changes committed and pushed
- [ ] Tag created and pushed
- [ ] GitHub Actions workflow succeeded
- [ ] Release created on GitHub
- [ ] Binaries uploaded
- [ ] Homebrew formula updated
- [ ] Installation tested

## Platform Support

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux | x86_64 | ✅ Supported |
| Linux | aarch64 | ✅ Supported |
| macOS | x86_64 (Intel) | ✅ Supported |
| macOS | aarch64 (Apple Silicon) | ✅ Supported |

## Distribution Channels

1. **GitHub Releases** - Pre-built binaries
2. **Homebrew** - `brew install mr-kelly/tap/mato`
3. **Install Script** - `curl ... | bash`
4. **Cargo** - `cargo install mato` (future)
5. **APT/YUM** - Package repositories (future)

## Future: APT/YUM Packages

To add APT/YUM support, we'll need:

1. **Debian Package (.deb)**
   - Create `debian/` directory with control files
   - Use `cargo-deb` to build .deb packages
   - Host on packagecloud.io or custom repository

2. **RPM Package (.rpm)**
   - Create `.spec` file
   - Use `cargo-rpm` to build .rpm packages
   - Host on packagecloud.io or custom repository

3. **GitHub Actions Integration**
   - Add steps to build .deb and .rpm in release workflow
   - Upload to package repositories

Example workflow addition:
```yaml
- name: Build Debian package
  run: cargo deb --target ${{ matrix.target }}

- name: Build RPM package
  run: cargo rpm build --target ${{ matrix.target }}
```

## Notes

- Replace `mr-kelly` with your actual GitHub username in all files
- Update repository URLs in README.md, install.sh, and tap `Formula/mato.rb`
- Consider using GitHub Releases API for automated Homebrew formula updates
