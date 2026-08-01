# Bumps the project version in one command instead of manually syncing
# Cargo.toml / Cargo.lock / CHANGELOG.md (the exact kind of manual sync
# that caused installer/Cargo.toml versions to drift in the past).
#
# Does NOT commit, tag, or push - it only edits files. Review the diff,
# fill in the CHANGELOG section it creates, then commit/tag/push yourself.
#
# Usage: .\scripts\bump-version.ps1 -Version 0.6.0

param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

$SemverPattern = '^\d+\.\d+\.\d+$'
if ($Version -notmatch $SemverPattern) {
    Write-Error "Version '$Version' is not a valid semver (expected X.Y.Z, e.g. 0.6.0)"
    exit 1
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$CargoTomlPath = Join-Path $ProjectRoot "Cargo.toml"
$ChangelogPath = Join-Path $ProjectRoot "CHANGELOG.md"

if (-not (Test-Path $CargoTomlPath)) {
    Write-Error "Cargo.toml not found at $CargoTomlPath"
    exit 1
}

# Read as UTF-8 explicitly - PowerShell 5.1's Get-Content -Raw defaults to
# the system codepage, which mangles the non-ASCII author name in
# Cargo.toml (e.g. "Hai Nguyen" with diacritics) into mojibake on write-back.
$CargoTomlContent = [IO.File]::ReadAllText($CargoTomlPath, [Text.Encoding]::UTF8)
if ($CargoTomlContent -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    Write-Error "Could not find a top-level version in Cargo.toml"
    exit 1
}
$CurrentVersion = $Matches[1]

# Compare as [version] so "0.6.0" > "0.5.2" etc. is a real semantic
# comparison, not a string comparison.
try {
    $current = [version]$CurrentVersion
    $target = [version]$Version
} catch {
    Write-Error "Failed to parse versions for comparison: current='$CurrentVersion' target='$Version'"
    exit 1
}

if ($target -le $current) {
    Write-Error "New version ($Version) must be greater than the current version ($CurrentVersion)"
    exit 1
}

Write-Host "Bumping version: $CurrentVersion -> $Version" -ForegroundColor Cyan
Write-Host ""

# --- Cargo.toml ---
# Replace only the first top-level "version = ..." line (the package
# version), not any dependency's pinned version.
$updatedCargoToml = $CargoTomlContent -replace '(?m)^version\s*=\s*"[^"]+"', "version = `"$Version`""
[IO.File]::WriteAllText($CargoTomlPath, $updatedCargoToml, [Text.UTF8Encoding]::new($false))
Write-Host "Updated Cargo.toml" -ForegroundColor Green

# --- Cargo.lock ---
# Let cargo resolve the lockfile update itself rather than hand-editing it
# (guarantees a valid lockfile, avoids drift from the checksum/dep graph).
# Cargo prints ordinary build warnings to stderr; under $ErrorActionPreference
# = "Stop" that gets treated as a terminating error even on success, so
# relax it for just this call and check $LASTEXITCODE instead (the actual
# pass/fail signal). Do not redirect stderr - it's already visible, and
# redirecting it is what wraps each line in a distracting error record.
Push-Location $ProjectRoot
try {
    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    cargo check --quiet
    $ErrorActionPreference = $previousErrorActionPreference
    if ($LASTEXITCODE -ne 0) {
        Write-Error "cargo check failed while syncing Cargo.lock - fix build errors before bumping version"
        exit 1
    }
} finally {
    Pop-Location
}
Write-Host "Updated Cargo.lock (via cargo check)" -ForegroundColor Green

# --- CHANGELOG.md ---
if (-not (Test-Path $ChangelogPath)) {
    Write-Error "CHANGELOG.md not found at $ChangelogPath"
    exit 1
}

$changelogLines = [IO.File]::ReadAllText($ChangelogPath, [Text.Encoding]::UTF8) -split "`r?`n"
$today = Get-Date -Format "yyyy-MM-dd"

$unreleasedIndex = -1
for ($i = 0; $i -lt $changelogLines.Count; $i++) {
    if ($changelogLines[$i] -match '^## \[Unreleased\]' -or $changelogLines[$i] -match "^## \[$([regex]::Escape($Version))\] - Unreleased") {
        $unreleasedIndex = $i
        break
    }
}

if ($unreleasedIndex -ge 0) {
    # Turn "## [Unreleased]" or "## [X.Y.Z] - Unreleased" into a dated entry.
    $changelogLines[$unreleasedIndex] = "## [$Version] - $today"
    Write-Host "Dated existing [Unreleased] section as $Version - $today" -ForegroundColor Green
} else {
    # No existing section for this version - insert a fresh dated section
    # with empty headings right after the title/intro (before the first
    # "## [" entry), so bump-version can be re-run safely across releases.
    $firstSectionIndex = 0
    for ($i = 0; $i -lt $changelogLines.Count; $i++) {
        if ($changelogLines[$i] -match '^## \[') {
            $firstSectionIndex = $i
            break
        }
    }

    $newSection = @(
        "## [$Version] - $today"
        ""
        "### Added"
        ""
        "### Changed"
        ""
        "### Fixed"
        ""
    )

    $before = $changelogLines[0..($firstSectionIndex - 1)]
    $after = $changelogLines[$firstSectionIndex..($changelogLines.Count - 1)]
    $changelogLines = $before + $newSection + $after
    Write-Host "Inserted new CHANGELOG section for $Version - fill it in before releasing" -ForegroundColor Yellow
}

$updatedChangelog = ($changelogLines -join "`n") + "`n"
[IO.File]::WriteAllText($ChangelogPath, $updatedChangelog, [Text.UTF8Encoding]::new($false))

Write-Host ""
Write-Host "Done. Next steps:" -ForegroundColor Cyan
Write-Host "  1. Review CHANGELOG.md and fill in the $Version section" -ForegroundColor Gray
Write-Host "  2. git add -A && git commit -m `"chore: bump version to $Version`"" -ForegroundColor Gray
Write-Host "  3. git tag v$Version && git push origin main --tags" -ForegroundColor Gray
