# Deployment Guide

How to ship a new Windows release of MechvibesDX, end to end.

## Prerequisites (one-time, per machine)

- Rust toolchain (`rustup`) and Cargo on `PATH`.
- [Inno Setup 6](https://jrsoftware.org/isinfo.php) installed, for local installer builds. CI installs it automatically via `choco install innosetup`.
- Push access to `main` and permission to create tags on the repo.

## Release steps

1. **Bump the version**

   ```powershell
   .\scripts\bump-version.ps1 -Version 0.6.0
   ```

   This updates `Cargo.toml`, regenerates `Cargo.lock` (via `cargo check`), and either dates the existing `## [Unreleased]` CHANGELOG section or inserts a new empty `## [0.6.0] - <date>` section. It does **not** commit anything.

2. **Fill in the CHANGELOG**

   Edit `CHANGELOG.md` and replace the empty `### Added` / `### Changed` / `### Fixed` headings under the new version section with real, user-facing entries. This becomes the GitHub release's notes verbatim (via `extract-changelog.ps1`), so write it for end users, not for other developers.

3. **Commit**

   ```powershell
   git add -A
   git commit -m "chore: bump version to 0.6.0"
   ```

4. **Tag and push**

   ```powershell
   git tag v0.6.0
   git push origin main --tags
   ```

   Pushing the `v*` tag triggers `.github/workflows/release.yml`.

5. **Wait for GitHub Actions**

   The workflow (~10-15 minutes): validates the tag matches `Cargo.toml`'s version, extracts the CHANGELOG section as release notes, runs `cargo test --release` + `cargo check --release` as a smoke test, builds the release binary, builds the installer (via `scripts/build-windows-installer.ps1`, same script used locally), and publishes a **draft** GitHub release with the installer attached.

6. **Review the draft release**

   Go to the repo's Releases page, open the draft, and sanity-check the notes and the attached `MechvibesDX-<version>-Setup-x64.exe`. Download and install it on a real Windows machine if this is a release you're not fully confident in.

7. **Publish**

   Click "Publish release" on the draft. The in-app auto-updater (`src/utils/auto_updater.rs`) reads published (non-draft) releases from the GitHub API, so existing installs will detect the update after this step - not before.

## Why a draft first?

Draft releases don't appear in the public `GET /releases` API response that the app's auto-updater polls, so a bad build never reaches existing users automatically. Publishing is a deliberate, separate action.

## Troubleshooting

**Workflow fails at "Gate - tag must match Cargo.toml version"**
The tag (e.g. `v0.6.0`) doesn't match the `version` in `Cargo.toml`. Either you tagged before bumping the version, or the bump didn't get committed before tagging. Delete the tag (`git tag -d v0.6.0 && git push origin :refs/tags/v0.6.0`), fix the version, re-tag.

**Workflow fails at "Gate - extract changelog notes"**
`CHANGELOG.md` has no `## [<version>]` section, or the section still contains the word "Unreleased" (a placeholder, not a real release date). Run `bump-version.ps1` again or edit the CHANGELOG heading by hand to `## [X.Y.Z] - YYYY-MM-DD`.

**`cargo test` or `cargo check` fails in the smoke test step**
The workflow does not build an installer or create a release if this fails. Fix the underlying test/build failure - do not bypass this gate.

**"No installer asset found" / asset name doesn't match the auto-updater filter**
The auto-updater only recognizes Windows assets whose filename contains `x64` and ends in `.exe`. If you renamed the `.iss` script's `OutputBaseFilename`, keep that constraint (see `src/utils/auto_updater.rs`'s `find_download_url`).

**Local installer build fails with "Inno Setup not found"**
Install Inno Setup 6 from https://jrsoftware.org/isinfo.php, or if already installed via winget, confirm it's at one of the paths `scripts/build-windows-installer.ps1` checks (`Program Files\Inno Setup 6`, or `%LOCALAPPDATA%\Programs\Inno Setup 6` for a per-user winget install).

## Scope note

This pipeline currently covers **Windows only**. Linux packaging (`.deb`/AppImage) is being redesigned separately and isn't part of this release workflow yet.
