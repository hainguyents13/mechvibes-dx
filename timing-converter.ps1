#!/usr/bin/env pwsh
# Wrapper script to run timing converters from root directory

param(
    [Parameter(Position=0)]
    [ValidateSet("single", "batch", "help")]
    [string]$Mode = "help",
    
    [Parameter(Position=1)]
    [string]$ConfigPath = ""
)

if ($Mode -eq "help") {
    Write-Host "🔧 MechvibesDX Timing Converter" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\timing-converter.ps1 single <config-path>   # Convert single soundpack"
    Write-Host "  .\timing-converter.ps1 batch                  # Convert all soundpacks"
    Write-Host "  .\timing-converter.ps1 help                   # Show this help"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Green
    Write-Host "  .\timing-converter.ps1 single soundpacks\keyboard\cherrymx-black-abs\config.json"
    Write-Host "  .\timing-converter.ps1 batch"
    return
}

if ($Mode -eq "single") {
    if ([string]::IsNullOrEmpty($ConfigPath)) {
        Write-Error "Config path is required for single mode"
        Write-Host "Usage: .\timing-converter.ps1 single <config-path>"
        exit 1
    }
    
    Write-Host "🔧 Converting single soundpack: $ConfigPath" -ForegroundColor Cyan
    & "utils\convert-timing.ps1" -ConfigPath $ConfigPath
}
elseif ($Mode -eq "batch") {
    Write-Host "🔧 Converting all soundpacks..." -ForegroundColor Cyan
    & "utils\batch-convert-timing.ps1"
}
