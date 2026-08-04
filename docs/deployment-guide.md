# Deployment Guide

How to ship a new release of MechvibesDX, end to end.

Windows is the primary target: it is the only platform with an installer and the only one the in-app auto-updater serves. Linux and macOS binaries ride along on the same tag.

## Prerequisites (one-time, per machine)

- Rust toolchain (`rustup`) and Cargo on `PATH`.
- [Inno Setup 6](https://jrsoftware.org/isinfo.php) installed, for local installer builds. CI installs it automatically via `choco install innosetup`.
- Push access to `main` and permission to create tags on the repo.
- Activate the repo's local quality gates (versioned git hooks):

  ```powershell
  git config core.hooksPath .githooks
  ```

## Local gates (every commit and push)

The `.githooks/` hooks enforce that nothing broken leaves your machine:

- **pre-commit** runs `cargo check` — the tree must compile before any commit.
- **pre-push** runs `cargo test` — the full suite must pass before anything reaches origin (mirrors the CI smoke gate, so a red push never wastes a CI run or lands on another session's clone).

`--no-verify` bypasses them in an emergency, but CI runs the same gates on release tags, so a bypassed failure only postpones the red light.

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

   The workflow runs four jobs (~15 minutes end to end):

   - **`windows-build`** — validates the tag matches `Cargo.toml`'s version, extracts the CHANGELOG section as release notes, runs `cargo test --release` + `cargo check --release` as a smoke test, builds the release binary, and builds the installer via `scripts/build-windows-installer.ps1` (the same script used locally).
   - **`linux-build`** — builds the release binary on `ubuntu-latest`, packages a `.deb` (via `cargo-deb`, driven by `[package.metadata.deb]` in `Cargo.toml`), then assembles an AppImage from the same binary via `scripts/build-linux-appimage.sh`. That order is verified safe: `cargo deb --no-build` leaves `target/release/mechvibes-dx` unstripped and byte-identical (same BuildID lands inside the AppImage).
   - **`macos-build`** — builds the release binary, then hand-assembles `MechvibesDX.app` and packages it as a DMG via `scripts/build-macos-app.sh`. Marked `continue-on-error`, so a red macOS job never blocks the release.
   - **`release`** — downloads all three jobs' artifacts and creates one **draft** release with everything attached.

   The build jobs run in parallel and each uploads artifacts; only the final `release` job writes to GitHub Releases. That single-writer design is deliberate — having each job call `action-gh-release` against the same tag races, and the last writer can drop the others' assets.

   `release` requires `windows-build` to succeed but tolerates the other two failing, so a Linux or macOS regression degrades the release rather than blocking it.

6. **Review the draft release**

   Go to the repo's Releases page, open the draft, and check the notes and assets:

   | Asset | Platform | Notes |
   |---|---|---|
   | `MechvibesDX-<version>-Setup-x64.exe` | Windows | The installer. **The only asset the auto-updater consumes** — its name must keep containing `x64` and ending in `.exe`. |
   | `mechvibes-dx_<version>_amd64.deb` | Debian/Ubuntu | `sudo dpkg -i`. Does **not** add the user to the `input` group (see below). |
   | `mechvibes-dx-<version>-x86_64.AppImage` | Any Linux distro | Portable, no install. Needs `chmod +x` first, and has the **same** `input` group requirement as the `.deb`. |
   | `mechvibes-dx-<version>-macos-<arch>-experimental.dmg` | macOS | **Experimental, ad-hoc signed, not notarized, untested.** Contains `MechvibesDX.app` and an `/Applications` symlink. |
   | `README-macos-<version>.txt` | macOS | DMG install, Gatekeeper and Accessibility instructions. |
   | `SHA256SUMS.txt` | all | Digests of every other asset. **The in-app auto-updater refuses to run an installer that is not listed here with a matching hash**, so a release missing this file silently degrades every Windows user to a manual download. |

   Download and install the Windows installer on a real machine if this is a release you're not fully confident in.

   **`SHA256SUMS.txt` is what makes silent auto-update possible.** The `release` job generates it with `sha256sum *` over the collected assets and then asserts the Windows installer is covered, so it cannot go out empty or stale. The app downloads it from the same release, finds the line for the installer, and verifies the file before executing anything; a mismatch deletes the download. Note the honest limit of this: the hash and the binary come from the same GitHub release, so it protects against corrupted transfers and tampering in flight, **not** against a compromised repository. Code signing remains the open item for that.

   Releases published before this step existed (v0.6.1 and earlier) have no `SHA256SUMS.txt`. That is handled, not broken: the app reports "checksums unavailable" and falls back to the browser download button, which is the pre-Phase-6 behavior.

   **Neither Linux artifact configures input permissions.** `maintainer-scripts` was deliberately removed from `[package.metadata.deb]`, so installing the package never modifies the user's groups, and an AppImage has no install step that could do so even in principle. Linux users must run `sudo usermod -a -G input $USER` and start a new session themselves, or the app runs silently. The generated release notes say this for both assets; keep that wording if you touch them. (The portable tarball was dropped by user decision 260803; the AppImage now covers the non-Debian users that decision left building from source.)

   **The AppImage's AppDir mirrors the `.deb` layout** (`usr/bin`, `usr/share/mechvibes-dx/soundpacks`) so a single binary serves both packages. Two placements are load-bearing, not cosmetic:

   - `usr/lib/mechvibes-dx/assets` — `dioxus-asset-resolver`'s Linux branch scans `<exe>/../../lib/<dir>/` for a directory containing `assets/`. Put the fonts anywhere else and every `asset!()` font silently 404s.
   - The `.desktop` file and icon are duplicated at the **AppDir root**, which `appimagetool` requires; the copies under `usr/share` are what a desktop-integration helper installs later.

   `src/state/paths.rs` resolves soundpacks **relative to the mount point** when running from an AppImage, and that ordering matters: the absolute `/usr/share/mechvibes-dx/soundpacks` fallback is a real directory on a machine that also has the `.deb` installed, so checking it first would make a running AppImage silently load the other installation's soundpacks. Writable state goes to `~/.local/share/mechvibes/data` for the same reason it does on macOS — the image is mounted read-only, and its mount point is a fresh temporary directory on every launch.

   The job asserts bundled soundpack audio, `config.json` and font counts **match the source tree** (same rationale as macOS: the mouse packs are `.mp3`, so an `.ogg`-only check would pass while dropping all four), unpacks the finished image with `--appimage-extract` and checks the ELF magic byte-wise. `appimagetool` runs with `APPIMAGE_EXTRACT_AND_RUN=1` because GitHub runners have no FUSE, so it cannot mount itself.

   **macOS assets are labelled experimental on purpose.** The build has never been run on a real Mac and is **not notarized**, so Gatekeeper still blocks a plain double-click (right-click → Open clears it, once). The label belongs in both the filename and the notes so nobody mistakes it for a supported download.

   **The macOS asset is a DMG containing a hand-assembled `.app`.** `dx bundle` is not used — DioxusLabs/dioxus#5723 makes the 0.7.x resource copier fail on every directory in `Dioxus.toml`'s `resources`, and it exits non-zero having already written a `.app` with an *empty* `Contents/Resources`, so "use the .app if one exists" would ship a silent app. `scripts/build-macos-app.sh` assembles the bundle explicitly (`Info.plist`, binary, `Resources/` with soundpacks + assets + a generated `.icns`), ad-hoc signs it (`codesign --force --deep -s -`), and packages it with `hdiutil`. Revisit once the upstream fix ships (present in 0.8.0-alpha.1).

   The job asserts the bundled soundpack audio and `config.json` counts **match the source tree** rather than merely being non-zero — the mouse packs are `.mp3`, so an `.ogg`-only check would pass while silently dropping all four. `spctl` is run for information but deliberately **not** gated on: it fails without notarization, which is the expected state.

   **Two path rules the bundle depends on** (`src/state/paths.rs`): resources resolve to `Contents/Resources` when the executable sits in `<name>.app/Contents/MacOS/`, which is also where `dioxus-asset-resolver` hardcodes the asset root on macOS — hence `assets/` is copied into `Resources/` alongside `soundpacks/`, or every `asset!()` font 404s. And writable state (`config.json`, `themes.json`, the cache) moves to `~/Library/Application Support/Mechvibes/data` inside a bundle, because an app in `/Applications` is not user-writable and settings would otherwise silently never persist.

7. **Publish**

   Click "Publish release" on the draft. The in-app auto-updater (`src/utils/auto_updater.rs`) reads published (non-draft) releases from the GitHub API, so existing installs will detect the update after this step - not before.

   Publishing **automatically announces** the release to the Mechvibes Discord's release channel (verified live with v0.7.0) — a Mantine-style message (greeting + release notes + links) with an `@everyone` ping. Nothing to run for a normal release.

   Manual runs of `announce-release.yml` are for exceptional cases only, and **publishing already posts on its own — a manual run about the same release creates a second message** (this happened on v0.7.0; one had to be deleted by hand):

   ```powershell
   gh workflow run announce-release.yml -f tag=vX.Y.Z      # re-announce an already-published release (recovery)
   gh workflow run announce-release.yml -f message="..."   # hand-written announcement, posted verbatim with a real ping
   gh workflow run announce-release.yml                    # ping-free test message
   ```

   The `message` mode adds nothing around your text and fails (rather than truncates) past Discord's 2000-character cap. A historical note so nobody re-learns this the hard way: v0.6.3 was not announced automatically simply because this workflow **did not exist yet** when that release was published — not because of any `GITHUB_TOKEN` event rule; GitHub's anti-recursion rule only suppresses events *performed by* the token, and clicking Publish is performed by you. One-time setup: create a webhook in the channel (Channel settings → Integrations → Webhooks), then `gh secret set DISCORD_RELEASE_WEBHOOK`. No bot or bot token involved.

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

Note the same filter constrains the *other* platforms' asset names in reverse: no Linux or macOS asset may contain `x64` and end in `.exe`, or it could be served to Windows users as an update. Both jobs assert this before uploading, and the names use `x86_64`/`amd64`/`arm64` (none of which contain the substring `x64`) with non-`.exe` extensions. If the filter in `find_download_url` ever loosens, revisit those assertions.

**`linux-build` fails on a missing `-dev` package**
Its system-dependency list mirrors `ci.yml`'s `linux-check` job. If you add a dependency to one, add it to the other — otherwise CI stays green while releases break.

**`macos-build` is red**
Expected to be possible: the crate's macOS support is unverified. The job is `continue-on-error`, so the release still goes out with Windows and Linux assets and simply omits the macOS one. Fix it or leave it; it does not gate a release.

**Local installer build fails with "Inno Setup not found"**
Install Inno Setup 6 from https://jrsoftware.org/isinfo.php, or if already installed via winget, confirm it's at one of the paths `scripts/build-windows-installer.ps1` checks (`Program Files\Inno Setup 6`, or `%LOCALAPPDATA%\Programs\Inno Setup 6` for a per-user winget install).

## Scope note

Windows is the only fully supported platform: installer, upgrade path, and in-app auto-update.

Linux gets a working `.deb` **and** an AppImage, but no automated input-group setup and no auto-update on either. The AppImage is built by `scripts/build-linux-appimage.sh`, written fresh for this pipeline; the legacy `scripts/build-linux-installer.sh` referenced by older docs no longer exists and was never verified.

## How in-app update install works (Windows)

Windows is the only platform with an unattended upgrade path. **Nothing is ever downloaded automatically** — a version check only notifies, and the ~50 MB transfer starts solely when the user clicks "Download & install" in Settings. There is no auto-download setting, because downloading is already an explicit act.

1. A check (at startup, or on the 24h tick) finds a newer non-prerelease tag and shows a badge in the title bar. The badge routes to Settings; it does **not** open the raw `.exe` in a browser, which would bypass verification.
2. The user clicks **"Download & install vX.Y.Z"**. The button then carries the whole state machine: `Downloading new version...` → `Restart to finish update` / `Later` → `Installing...`.
3. The app downloads `SHA256SUMS.txt` **first**. No checksums, no download — this costs one small request instead of ~50 MB that would have to be discarded.
4. The installer streams to `%TEMP%\mechvibes-updates\<name>.exe.partial`, is verified against its digest, and only then renamed to its final name. An interrupted download can never be mistaken for a finished one.
5. **Restart to finish update** re-verifies once more, clears the staged entry from config, spawns the installer detached with `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`, and closes the app. **Later** keeps the verified file, so returning to the button — or restarting the app — offers the install again without downloading a second time (the hash is re-checked then, so a file altered in between is discarded rather than run).

Two details in `installer/windows/mechvibes-dx-setup.iss` that the update flow depends on — do not remove them without reading this:

- `CloseApplications=yes` lets Setup close the running app that holds `mechvibes-dx.exe`.
- The second `[Run]` entry (`skipifnotsilent`, no `Check:`) is the **only** thing that relaunches the app after a silent install. Do not add a `Check: not RmSessionStarted` guard to it — that was tried and it broke auto-update entirely (app closed, never returned). `CloseApplications=yes` uses Restart Manager to close us, so a session always exists and that Check is always false; meanwhile RM itself only revives processes that called `RegisterApplicationRestart`, which this app never does. A real `/LOG` trace shows RM logging "Attempting to restart applications" and nothing coming back. Double-launch is not a concern: the app's single-instance mutex makes a second copy exit immediately.

Everything in this flow fails soft. A network error, a missing `SHA256SUMS.txt`, a hash mismatch, or a blocked installer all leave the app running; the button shows a short reason and an "Open download page" link, which is the pre-Phase-6 behavior.

macOS is experimental: ad-hoc signed but not notarized, and never run on real hardware by the maintainer. It ships as a `.dmg` so users can drag-install and report back.
