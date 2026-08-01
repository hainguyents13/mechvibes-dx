# Extracts the release-notes body for one version from CHANGELOG.md.
# Prints the section content (everything between "## [<Version>]" and the
# next "## [" heading, or end of file) to stdout, with the heading line
# itself stripped. Used both locally and by the release CI to populate
# GitHub release notes from a single source of truth.
#
# Usage: .\scripts\extract-changelog.ps1 -Version 0.5.2
# Exit code 0 on success (section found and printed).
# Exit code 1 if CHANGELOG.md is missing, the version has no section, or
# the section still contains the placeholder word "Unreleased" (a release
# must have a real, dated changelog entry).

param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$ChangelogPath = Join-Path $ProjectRoot "CHANGELOG.md"

if (-not (Test-Path $ChangelogPath)) {
    Write-Error "CHANGELOG.md not found at $ChangelogPath"
    exit 1
}

# Read as UTF-8 explicitly - PowerShell 5.1's Get-Content defaults to the
# system codepage for non-ASCII text.
$lines = [IO.File]::ReadAllText($ChangelogPath, [Text.Encoding]::UTF8) -split "`r?`n"

$startIndex = -1
for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match "^## \[$([regex]::Escape($Version))\]") {
        $startIndex = $i
        break
    }
}

if ($startIndex -eq -1) {
    Write-Error "No CHANGELOG.md section found for version $Version (expected a line starting with '## [$Version]')"
    exit 1
}

if ($lines[$startIndex] -match "Unreleased") {
    Write-Error "CHANGELOG.md section for $Version still says 'Unreleased' - update it with the real release date before releasing"
    exit 1
}

$endIndex = $lines.Count - 1
for ($i = $startIndex + 1; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match "^## \[") {
        $endIndex = $i - 1
        break
    }
}

$body = $lines[($startIndex + 1)..$endIndex] -join "`n"
$body = $body.Trim()

if ([string]::IsNullOrWhiteSpace($body)) {
    Write-Error "CHANGELOG.md section for $Version is empty"
    exit 1
}

Write-Output $body
